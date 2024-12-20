use owo_colors::OwoColorize;
pub struct LogColors {}

impl LogColors {
    pub fn purple(text: &str) -> String {
        // use #4B3978 as purple color

        text.purple().to_string()
    }

    pub fn green(text: &str) -> String {
        // use #04cd9b as green color

        text.green().to_string()
    }

    pub fn alert(text: &str) -> String {
        // use #FF0000 as red color
        text.red().to_string()
    }
}
