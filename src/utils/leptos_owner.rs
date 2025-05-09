/// Utility to safely execute a closure with the current Leptos owner.
/// If the owner is disposed, logs and returns None.
pub fn with_owner_safe<F, R>(log_context: &str, f: F) -> Option<R>
where
    F: FnOnce() -> R,
{
    if let Some(owner) = leptos::Owner::current() {
        leptos::try_with_owner(owner, f).ok()
    } else {
        leptos::logging::log!("[OWNER] No Leptos owner in context: {}", log_context);
        None
    }
}