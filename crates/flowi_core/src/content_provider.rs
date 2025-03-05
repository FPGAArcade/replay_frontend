use crate::io::io::IoHandle;
use crate::Ui;

#[derive(Debug, Copy, Clone)]
pub struct Item {
    /// The background image of the item. This is the image that is shown in the background of the
    /// item. Image is scaled to power of two.
    pub background_image: IoHandle,
    /// This image is being shown when the item is non-selected. We used a scaled down image
    /// that fits the screen size we need exactly to save performance.
    pub image: IoHandle,
    /// The ID of the item. This is used to identify the item in the content provider.
    pub id: u64
}

/// The visibility of an item. This is used to determine how the item should be displayed.
/// It's also useful to hint to the loading system to priority load items that are visible.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ItemVisibility {
    Hidden,
    Visible,
    Selected,
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
    fn get_item_id(&mut self, row: u64, col: u64) -> u64;
    fn get_item(&mut self, ui: &Ui, visible: ItemVisibility, row: u64, col: u64) -> Item;
    fn get_column_count(&mut self, ui: &Ui, row: u64) -> u64;
    /// Get the name of the row
    fn get_row_name(&mut self, ui: &Ui, row: u64) -> &str;
}
