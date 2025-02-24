use swayipc::{Connection, Output};
// use serde::Serialize;
// use serde_json::json;

const BUILTIN_OUTPUT_NAME: &str = "eDP-1";

enum Orientation {
    Above,
    Below,
    Left,
    Right
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
        Display{
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

        if self.width > other.width {
            other_x_offset = (self.width - other.width) / 2;
        } else if self.width < other.width {
            self_x_offset = (other.width - self.width) / 2;
        }

        if self.height > other.height {
            other_y_offset = (self.height - other.height) / 2;
        } else if self.height < other.height {
            self_y_offset = (other.height - self.height) / 2;
        }

		match orientation {
    		Orientation::Below => {
        		other.x = other_y_offset;
        		other.y = 0;
        		self.x = self_x_offset;
        		self.y = other.height;
    		},
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

fn main() {
    let mut connection = Connection::new().expect("Error creating connection");
    let outputs = connection.get_outputs().expect("Error getting outputs");
    if outputs.len() <= 1 {
        return;
    }

	// TODO: Accept from input or config
    let orientation = Orientation::Below;

    let main_output = outputs.iter().find(|output| { output.name == BUILTIN_OUTPUT_NAME} ).ok_or(format!("No output found with name {}", BUILTIN_OUTPUT_NAME)).unwrap();
    let secondary_output = outputs.iter().find(|output| { output.name != BUILTIN_OUTPUT_NAME} ).unwrap();

    let mut main_display = Display::new(main_output);
    let mut secondary_display = Display::new(secondary_output);

    main_display.place(orientation, &mut secondary_display);

	[main_display, secondary_display].into_iter().for_each(|display| {
    	println!("{:?}", connection.run_command(display.to_command_str()));
	});


}
