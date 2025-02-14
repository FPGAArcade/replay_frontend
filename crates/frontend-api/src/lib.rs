use image::RenderImage;

struct Item {
    /// This image is being shown when the item is non-selected. We used a scaled down image
    /// that fits the screen size we need exactly to save performance.
    unselected_image: Option<RenderImage>,
    /// This image is being shown when the item is selected. This has the original size when
    /// loaded from the source, unless it's very large it will have been downsized as well.
    selected_image: Option<RenderImage>,
    /// The ID of the item. This is used to identify the item when it's selected.
    id: u64,
}

/// The content provider is responsible for providing the content to the content selector. This
/// is a trait that needs to be implemented by the user of the content selector. The content selector
/// will call these functions to get the data it needs to display the items. The content provider
/// is responsible for loading the data from the source and provide it to the content selector. The
/// idea is that the content selector should be as generic as possible and not have any knowledge
/// of the data source as we want to support demos, games, etc., from various sources.
trait ContentProvider {
    /// Get the item at the given column and row. If the item is not available at the given
    /// position it should return None.
    fn get_item(&self, row: u64, col: u64) -> Option<Item>;
    /// Get the number of columns in the grid
    fn get_row_count(&self, row: u64) -> u64;
    /// Get the name of the row
    fn get_row_name(&self, row: u64) -> &str;
}
