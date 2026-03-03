use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct AuthToken {
    /// The raw token bytes from your Alembic accounts system
    pub token: String,
}
