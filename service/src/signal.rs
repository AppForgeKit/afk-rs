use signal_hook::consts;
use std::ffi::c_int;
use std::fmt::Display;

#[derive(Clone)]
pub enum Signal {
    UNKNOWN,
    SIGINT,
    SIGTERM,
    #[cfg(unix)]
    SIGUSR_1,
    #[cfg(unix)]
    SIGUSR_2,
}

impl Signal {
    pub fn is_unknown(&self) -> bool {
        matches!(*self, Signal::UNKNOWN)
    }

    pub fn is_terminate(&self) -> bool {
        matches!(*self, Signal::SIGINT | Signal::SIGTERM)
    }
}

impl Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "signal: {}", Into::<c_int>::into(self))
    }
}

impl From<&Signal> for c_int {
    fn from(signal: &Signal) -> c_int {
        match signal {
            Signal::UNKNOWN => 0 as c_int,
            Signal::SIGINT => consts::SIGINT,
            Signal::SIGTERM => consts::SIGTERM,
            #[cfg(unix)]
            Signal::SIGUSR_1 => consts::SIGUSR_1,
            #[cfg(unix)]
            Signal::SIGUSR_2 => consts::SIGUSR_2,
        }
    }
}

impl From<&c_int> for Signal {
    fn from(signal: &c_int) -> Signal {
        match *signal {
            consts::SIGINT => Signal::SIGINT,
            consts::SIGTERM => Signal::SIGTERM,
            #[cfg(unix)]
            consts::SIGUSR_1 => Signal::SIGUSR_1,
            #[cfg(unix)]
            consts::SIGUSR_2 => Signal::SIGUSR_2,
            _ => Signal::UNKNOWN,
        }
    }
}

impl From<Signal> for c_int {
    fn from(signal: Signal) -> c_int {
        (&signal).into()
    }
}

impl From<c_int> for Signal {
    fn from(signal: c_int) -> Signal {
        (&signal).into()
    }
}

impl From<&Signal> for usize {
    fn from(signal: &Signal) -> usize {
        c_int::from(signal) as usize
    }
}

impl From<&usize> for Signal {
    fn from(signal: &usize) -> Signal {
        ((*signal) as c_int).into()
    }
}

impl From<Signal> for usize {
    fn from(signal: Signal) -> usize {
        (&signal).into()
    }
}

impl From<usize> for Signal {
    fn from(signal: usize) -> Signal {
        (&signal).into()
    }
}
