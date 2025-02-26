use std::cmp::Ordering;

use swayipc::{Connection, Output};

const BUILTIN_OUTPUT_NAME: &str = "eDP-1";

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
                other.x = other_y_offset;
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
                other.y = other_y_offset;
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

fn arange_outputs(main_output_name: &str, orientation: Orientation, secondary_output_name: Option<&str>) {
    let mut connection = Connection::new().expect("Error creating connection");
    let outputs = connection.get_outputs().expect("Error getting outputs");
    if outputs.len() <= 1 {
        return;
    }

    let main_output = outputs
        .iter()
        .find(|output| output.name == main_output_name)
        .expect("No main output found");
    let secondary_output = outputs
        .iter()
        .find(|output| {
            match secondary_output_name {
                Some(name) => output.name == name,
                None => output.name != main_output_name
            }
        })
        .unwrap();

    let mut main_display = Display::new(main_output);
    let mut secondary_display = Display::new(secondary_output);

    secondary_display.place(orientation, &mut main_display);

    [main_display, secondary_display]
        .into_iter()
        .for_each(|display| {
            connection.run_command(display.to_command_str()).expect("Error setting output") ;
        });
}

fn main() {
    arange_outputs(BUILTIN_OUTPUT_NAME, Orientation::Below, None);
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

        })).unwrap()
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
        let output1 = mock_output("main_test", 100, 100);
        let output2 = mock_output("secondary_test", 200, 200);
        let mut main_display = Display::new(&output1);
        let mut secondary_display = Display::new(&output2);

        main_display.place(Orientation::Above, &mut secondary_display);

        assert_eq!(main_display.x, (secondary_display.width - main_display.width) / 2);
        assert_eq!(main_display.y, 0);
        assert_eq!(secondary_display.x, 0);
        assert_eq!(secondary_display.y, main_display.height);
    }

    #[test]
    fn display_to_command_str() {
        let output1 = mock_output("main_test", 100, 100);
        let output2 = mock_output("secondary_test", 200, 200);
        let mut main_display = Display::new(&output1);
        let mut secondary_display = Display::new(&output2);

        main_display.place(Orientation::Above, &mut secondary_display);

        assert_eq!(main_display.to_command_str(), "output main_test pos 50 0");
        assert_eq!(secondary_display.to_command_str(), "output secondary_test pos 0 100");
    }
}
