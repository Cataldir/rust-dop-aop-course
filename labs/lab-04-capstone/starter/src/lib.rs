//! Intentional model exercise. Delegate typed proposals without mutating the original.
pub fn reduce(saved:&desktop_core::Saved,command:desktop_core::Command)->Result<desktop_core::Saved,desktop_core::UiError>{let _=(saved,command);Err(desktop_core::UiError::Invalid)}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn local_edit_then_undo(){let original=desktop_core::Saved::default();let added=reduce(&original,desktop_core::Command::Add{expected_revision:0,id:"cpu".into(),name:"CPU".into()}).unwrap();let undone=reduce(&added,desktop_core::Command::Undo{expected_revision:1}).unwrap();assert!(undone.view.panels.is_empty());assert_eq!(original.view.revision,0);}
    #[test]fn stale_revision_refuses(){assert!(matches!(reduce(&desktop_core::Saved::default(),desktop_core::Command::Undo{expected_revision:2}),Err(desktop_core::UiError::Stale)));}
}