pub mod editor;
pub mod preview;
pub mod sidebar;
pub mod toolbar;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Editor,
    Preview,
    Split,
}
