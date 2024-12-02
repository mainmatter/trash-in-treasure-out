use crate::types::ticket_machine::TicketMachine;

pub type Session = axum_session::Session<axum_session::SessionNullPool>;

const SESSION_STATE_KEY: &str = "STATE";

pub trait SessionExt {
    /// Get the state for this session, initializing it
    /// using [`TicketMachine::default`] if it doesn't
    /// exist
    fn get_or_init_state<F>(&self, f: F) -> TicketMachine
    where
        F: FnOnce(&mut TicketMachine);

    /// Update the session state if it exists, returning
    /// the updated state
    fn update_state<F>(&self, f: F) -> Option<TicketMachine>
    where
        F: FnOnce(&mut TicketMachine);

    /// Get the current state. Returns [`None`] if
    /// it doesn't exist for this session.
    fn try_get_state(&self) -> Option<TicketMachine>;
}

impl SessionExt for Session {
    fn get_or_init_state<F>(&self, f: F) -> TicketMachine
    where
        F: FnOnce(&mut TicketMachine),
    {
        self.try_get_state().unwrap_or_else(|| {
            self.set(SESSION_STATE_KEY, TicketMachine::default());
            self.try_get_state().unwrap()
        });

        self.update_state(f).unwrap()
    }

    fn update_state<F>(&self, f: F) -> Option<TicketMachine>
    where
        F: FnOnce(&mut TicketMachine),
    {
        self.try_get_state().map(|mut s| {
            f(&mut s);
            self.set(SESSION_STATE_KEY, s);
            self.try_get_state().unwrap()
        })
    }

    fn try_get_state(&self) -> Option<TicketMachine> {
        self.get(SESSION_STATE_KEY)
    }
}
