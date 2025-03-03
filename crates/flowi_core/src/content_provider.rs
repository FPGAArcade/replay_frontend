use crate::io::io::IoHandle;
use crate::Ui;

#[derive(Debug, Copy, Clone)]
pub struct Item {
    /// This image is being shown when the item is non-selected. We used a scaled down image
    /// that fits the screen size we need exactly to save performance.
    pub unselected_image: IoHandle,
    /// This image is being shown when the item is selected. This has the original size when
    /// loaded from the source, unless it's very large it will have been downsized as well.
    pub selected_image: IoHandle,
    /// The ID of the item. This is used to identify the item when it's selected.
    pub id: u64,
}

/// The content provider is responsible for providing the content to the content selector. This
/// is a trait that needs to be implemented by the user of the content selector. The content selector
/// will call these functions to get the data it needs to display the items. The content provider
/// is responsible for loading the data from the source and provide it to the content selector. The
/// idea is that the content selector should be as generic as possible and not have any knowledge
/// of the data source as we want to support demos, games, etc., from various sources.
pub trait ContentProvider {
    /// Get the item at the given column and row. If the item is not available at the given
    /// position it should return None.
    fn get_item(&mut self, ui: &Ui, row: u64, col: u64) -> Item;
    fn get_column_count(&mut self, ui: &Ui, row: u64) -> u64;
    /// Get the name of the row
    fn get_row_name(&mut self, ui: &Ui, row: u64) -> &str;
}
