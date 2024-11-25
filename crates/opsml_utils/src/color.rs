use colored::*;

pub struct LogColors {}

impl LogColors {
    pub fn purple(text: &str) -> String {
        // use #4B3978 as purple color
        let purple = Color::TrueColor {
            r: 75,
            g: 57,
            b: 120,
        };

        text.color(purple).to_string()
    }

    pub fn green(text: &str) -> String {
        // use #04cd9b as green color
        let green = Color::TrueColor {
            r: 4,
            g: 205,
            b: 155,
        };

        text.color(green).to_string()
    }

    pub fn alert(text: &str) -> String {
        // use #FF0000 as red color
        let red = Color::TrueColor { r: 255, g: 0, b: 0 };

        text.color(red).to_string()
    }
}
