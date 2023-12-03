#[cfg(test)]
mod tests {
    use crate::Image;
    use crate::ApplicationSettings;

    #[test]
    fn png_fail_load() {
        let settings = ApplicationSettings { some_data: 0 };
        let state = crate::FlowiState::new(&settings, 1);
        let handle = Image::create_from_file_block("fail_to_load.png");
        assert!(handle.is_err());
    }

    #[test]
    fn png_load_ok() {
        let settings = ApplicationSettings { some_data: 0 };
        let state = crate::FlowiState::new(&settings, 1);
        let handle = Image::create_from_file_block("/Users/emoon/code/projects/flowi/flowi_core/data/png/grayscale.png");
        handle.unwrap();
        //assert!(handle.is_ok());
    }
}
