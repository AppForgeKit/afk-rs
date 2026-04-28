use crate::Error;
use crate::signal;
use app_forge_kit_telemetry_tracing::debug;
use async_trait::async_trait;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::task::JoinSet;
use tokio::time;

#[async_trait]
pub trait Observable: Send + Sync {
    fn init(&self) -> Result<(), Error> {
        Ok(())
    }
    async fn serve(&self) -> Result<(), Error> {
        Ok(())
    }
    async fn signal(&self, _signal: &signal::Signal) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Default, PartialEq, Eq)]
enum ObservableState {
    #[default]
    Registered,
    Initialized,
    Started,
}

pub struct Resource {
    observable: Arc<Box<dyn Observable>>,
    state: RefCell<ObservableState>,
}

impl Resource {
    pub fn new(observable: Box<dyn Observable>) -> Self {
        Resource {
            observable: Arc::new(observable),
            state: RefCell::new(ObservableState::default()),
        }
    }
}

pub struct Observer {
    resources: RefCell<Vec<Resource>>,
    signals: RefCell<Vec<signal::Signal>>,
}

const OBSERVER_RESOURCES_CAPACITY: usize = 8;
const OBSERVER_SIGNALS_CAPACITY: usize = 4;

const SIGNAL_LOOP_SLEEP_DURATION_MICROS: u64 = 10;

#[allow(clippy::new_without_default)]
impl Observer {
    pub fn new() -> Self {
        let instance = Self {
            resources: RefCell::new(Vec::with_capacity(OBSERVER_RESOURCES_CAPACITY)),
            signals: RefCell::new(Vec::with_capacity(OBSERVER_SIGNALS_CAPACITY)),
        };

        instance
            .signals
            .borrow_mut()
            .extend([signal::Signal::SIGINT, signal::Signal::SIGTERM]);

        instance
    }

    pub fn register(&self, resource: Box<dyn Observable>) -> Result<(), Error> {
        self.resources.borrow_mut().push(Resource::new(resource));

        Ok(())
    }

    fn init(&self) -> Result<(), Error> {
        debug!("initializing");

        self.resources
            .borrow()
            .iter()
            .try_for_each(|resource| -> Result<(), Error> {
                resource.observable.init()?;
                resource.state.replace(ObservableState::Initialized);

                Ok(())
            })
    }

    fn register_signals_listeners(
        signals: &[signal::Signal],
        signal_catcher: &Arc<AtomicUsize>,
    ) -> Result<(), Error> {
        for signal in signals {
            signal_hook::flag::register_usize(
                signal.into(),
                signal_catcher.clone(),
                signal.into(),
            )?;
        }

        Ok(())
    }

    async fn signal_catcher(
        signals: &[signal::Signal],
        observables: &[Arc<Box<dyn Observable>>],
    ) -> Result<(), Error> {
        let signal_catcher = Arc::new(AtomicUsize::new(signal::Signal::UNKNOWN.into()));

        Self::register_signals_listeners(signals, &signal_catcher)?;

        let mut sleep_duration = time::Duration::from_micros(0);

        loop {
            if !sleep_duration.is_zero() {
                time::sleep(sleep_duration).await;
            }

            let signal_value: signal::Signal = signal_catcher.load(Ordering::Acquire).into();

            if signal_value.is_unknown() {
                if sleep_duration.is_zero() {
                    sleep_duration = time::Duration::from_micros(SIGNAL_LOOP_SLEEP_DURATION_MICROS);
                }

                continue;
            }

            debug!("catching signal: {}", signal_value);

            for observable in observables.iter() {
                observable.signal(&signal_value).await?;
            }

            if signal_value.is_terminate() {
                debug!("terminating");

                break;
            }
        }

        Ok(())
    }

    fn spawn_signal_catcher(&self, join_set: &mut JoinSet<Result<(), Error>>) -> Result<(), Error> {
        let signals: Arc<Vec<signal::Signal>> =
            Arc::new(self.signals.borrow().iter().cloned().collect());

        let observables: Arc<Vec<Arc<Box<dyn Observable>>>> = Arc::new(
            self.resources
                .borrow()
                .iter()
                .map(|resource| resource.observable.clone())
                .collect(),
        );

        join_set.spawn(async move { Self::signal_catcher(&signals, &observables).await });

        Ok(())
    }

    async fn join_handle(join_set: &mut JoinSet<Result<(), Error>>) -> Result<(), Error> {
        let mut result: Result<(), Error> = Ok(());

        while let Some(join_handle) = join_set.join_next().await {
            let mut need_shutdown = false;

            match join_handle {
                Ok(join_handle_result) => {
                    if let Err(err) = join_handle_result {
                        result = Err(err);
                        need_shutdown = true;
                    }
                }
                Err(err) => {
                    if err.is_panic() {
                        result = Err(err.into());
                    }
                    need_shutdown = true;
                }
            }

            if need_shutdown {
                join_set.join_next().await;
                break;
            }
        }

        result
    }

    pub async fn run(&self) -> Result<(), Error> {
        self.init()?;

        let mut join_set: JoinSet<Result<(), Error>> = JoinSet::new();

        debug!("starting");
        self.spawn_signal_catcher(&mut join_set)?;

        for resource in self.resources.borrow().iter() {
            let observable = Arc::clone(&resource.observable);

            join_set.spawn(async move { observable.serve().await });

            resource.state.replace(ObservableState::Started);
        }

        Self::join_handle(&mut join_set).await
    }
}
