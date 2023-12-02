#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Idle,
    Counting { count: u32 },
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Reset,
    TimerElapsed,
    ButtonPress,
}

impl State {
    pub fn update(&mut self, event: Event) {
        *self = match (*self, event) {
            (_, Event::Reset) => State::Idle,
            (State::Idle, Event::ButtonPress) => State::Counting { count: 0 },
            (State::Counting { count }, Event::TimerElapsed) => {
                if count < 32 {
                    State::Counting { count: count + 1 }
                } else {
                    State::Idle
                }
            }
            (state, _) => state,
        }
    }

    pub fn active(&self) -> bool {
        matches!(self, State::Counting { .. })
    }

    pub fn value(&self) -> u32 {
        if let &State::Counting { count } = self {
            count
        } else {
            0
        }
    }
}
