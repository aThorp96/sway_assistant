use std::cmp::Ordering;

use clap::{Parser, ValueEnum};
use swayipc::{Connection, Output};

const BUILTIN_OUTPUT_NAME: &str = "eDP-1";

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Orientation {
    Above,
    Below,
    Left,
    Right,
}

struct Display {
    output: Output,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Display {
    fn new(output: &Output) -> Display {
        Display {
            output: (*output).clone(),
            x: output.rect.x,
            y: output.rect.y,
            width: output.current_mode.unwrap().width,
            height: output.current_mode.unwrap().height,
        }
    }

    fn place(&mut self, orientation: Orientation, other: &mut Display) {
        println!(
            "Placing {} {:?} {}",
            self.output.name, orientation, other.output.name
        );

        let mut other_x_offset = 0;
        let mut other_y_offset = 0;
        let mut self_x_offset = 0;
        let mut self_y_offset = 0;

        match self.width.cmp(&other.width) {
            Ordering::Greater => {
                other_x_offset = (self.width - other.width) / 2;
            }
            std::cmp::Ordering::Less => {
                self_x_offset = (other.width - self.width) / 2;
            }
            _ => {}
        }

        match self.height.cmp(&other.height) {
            Ordering::Greater => {
                other_y_offset = (self.height - other.height) / 2;
            }
            Ordering::Less => {
                self_y_offset = (other.height - self.height) / 2;
            }
            _ => {}
        }

        match orientation {
            Orientation::Below => {
                other.x = other_x_offset;
                other.y = 0;
                self.x = self_x_offset;
                self.y = other.height;
            }
            Orientation::Above => {
                other.x = other_x_offset;
                other.y = self.height;
                self.x = self_x_offset;
                self.y = 0;
            }
            Orientation::Right => {
                other.x = 0;
                other.y = other_y_offset;
                self.x = other.width;
                self.y = self_y_offset;
            }
            Orientation::Left => {
                other.x = self.width;
                other.y = other_x_offset;
                self.x = 0;
                self.y = self_y_offset;
            }
        }
    }

    fn to_command_str(&self) -> String {
        let cmd = format!("output {} pos {} {}", self.output.name, self.x, self.y);
        println!("{}", cmd);
        cmd
    }
}

fn arrange_outputs(
    main_output_name: String,
    orientation: Orientation,
    secondary_output_name: Option<String>,
) {
    let mut connection = Connection::new().expect("Error creating connection");
    let outputs = connection.get_outputs().expect("Error getting outputs");
    if outputs.len() <= 1 {
        return;
    }






    let mut primary_display = outputs
        .iter()
        .find(|o| o.name == main_output_name && o.active)
        .map(Display::new)
        .expect("No main output found!");
    let mut secondary_display = outputs
        .iter()
        .find(|o| {
            o.name != main_output_name
                && secondary_output_name.as_ref().is_none_or(|n| o.name == *n)
        })
        .map(Display::new)
        .expect("No secondary output found!");

    secondary_display.place(orientation, &mut primary_display);

    [primary_display, secondary_display]
        .into_iter()
        .for_each(|display| {
            connection
                .run_command(display.to_command_str())
                .expect("Error setting output");
        });
}

/// Select some outputs adn arrange tehm accordingly
#[derive(Parser)]
#[command(name = "arrange_outputs")]
#[command(author, version, about)]
struct Cli {
    /// The orientation of the secondary output with respect to the primary output
    #[arg(value_enum)]
    orientation: Orientation,

    /// The output which is the focus of the orientation
    #[arg(default_value = BUILTIN_OUTPUT_NAME, short = 'p', long = "primary")]
    primary_output_name: String,

    /// The output which is the subject of orientation
    #[arg(default_value = None, short = 's', long = "secondary")]
    secondary_output_name: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    arrange_outputs(
        cli.primary_output_name,
        cli.orientation,
        cli.secondary_output_name,
    );
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn mock_output(name: &str, width: i32, height: i32) -> Output {
        serde_json::from_value(json!({
            "name": name,
            "current_mode": {
                "width": width,
                "height": height,
                "refresh": 0,
            },
            "make": "fake",
            "model": "fake",
            "serial": "",
            "active": true,
            "dpms": true,
            "primary": false,
            "modes": [],
            "rect": {
                "x": 0,
                "y": 0,
                "width": width,
                "height": height,
            },

        }))
        .unwrap()
    }

    #[test]
    fn display_initializes() {
        let output = mock_output("test", 100, 100);
        let display = Display::new(&output);
        assert_eq!(display.x, 0);
        assert_eq!(display.y, 0);
        assert_eq!(display.width, 100);
        assert_eq!(display.height, 100);
    }

    #[test]
    fn display_placement() {
        struct TestCase {
            orientation: Orientation,
            primary_x: i32,
            primary_y: i32,
            secondary_x: i32,
            secondary_y: i32,
        }

        let output1 = mock_output("main_test", 100, 100);
        let output2 = mock_output("secondary_test", 200, 200);
        let mut primary_display = Display::new(&output1);
        let mut secondary_display = Display::new(&output2);

        let test_cases = [
            TestCase {
                orientation: Orientation::Above,
                primary_x: (secondary_display.width - primary_display.width) / 2,
                primary_y: 0,
                secondary_x: 0,
                secondary_y: primary_display.height,
            },
            TestCase {
                orientation: Orientation::Below,
                primary_x: (secondary_display.width - primary_display.width) / 2,
                primary_y: secondary_display.height,
                secondary_x: 0,
                secondary_y: 0,
            },
            TestCase {
                orientation: Orientation::Left,
                primary_x: 0,
                primary_y: (secondary_display.height - primary_display.height) / 2,
                secondary_x: primary_display.width,
                secondary_y: 0,
            },
            TestCase {
                orientation: Orientation::Right,
                primary_x: secondary_display.height,
                primary_y: (secondary_display.height - primary_display.height) / 2,
                secondary_x: 0,
                secondary_y: 0,
            },
        ];

        test_cases.iter().for_each(|test_case| {
            primary_display.place(test_case.orientation, &mut secondary_display);

            assert_eq!(primary_display.x, test_case.primary_x);
            assert_eq!(primary_display.y, test_case.primary_y);
            assert_eq!(secondary_display.x, test_case.secondary_x);
            assert_eq!(secondary_display.y, test_case.secondary_y);
        });
    }

    #[test]
    fn display_to_command_str() {
        let output1 = mock_output("main_test", 100, 100);
        let output2 = mock_output("secondary_test", 200, 200);
        let mut main_display = Display::new(&output1);
        let mut secondary_display = Display::new(&output2);

        main_display.place(Orientation::Above, &mut secondary_display);

        assert_eq!(main_display.to_command_str(), "output main_test pos 50 0");
        assert_eq!(
            secondary_display.to_command_str(),
            "output secondary_test pos 0 100"
        );
    }
}
