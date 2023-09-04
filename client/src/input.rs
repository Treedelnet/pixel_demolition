use web_sys::HtmlCanvasElement;
use wasm_bindgen::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct Input {
    active_keys:Rc<Cell<[u32; Self::ACTIVE_KEYS_SIZE]>>,
    typed_keys:Rc<RefCell<Vec<u8>>>,
    mouse_clicked:Rc<Cell<bool>>,
    mouse_down:Rc<Cell<bool>>,
    mouse_coords:Rc<Cell<(i32, i32)>>,
    last_mouse_state:bool
}

impl Input {
    pub const ACTIVE_KEYS_SIZE: usize = 5;

    pub fn new() -> Input {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas.dyn_into().unwrap();

        console_error_panic_hook::set_once();

        // Track which keys are currently pressed
        let active_keys = Rc::new(Cell::new([0;Self::ACTIVE_KEYS_SIZE]));
        {
            let active_keys = active_keys.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
                let mut current_keys=active_keys.get();
                // Check if it is already stored
                for i in 0..Self::ACTIVE_KEYS_SIZE {
                    if current_keys[i]==event.key_code() {
                        return;
                    }
                }

                // Try to find a place in the current keys buffer to store it
                for i in 0..Self::ACTIVE_KEYS_SIZE {
                    if current_keys[i]==0  {
                        current_keys[i]=event.key_code();
                        break;
                    }
                        
                }
                active_keys.set(current_keys);
            });
            document
                .add_event_listener_with_callback("keydown", closure.as_ref()
                .unchecked_ref())
                .unwrap();

            closure.forget();
        }
        {
            let active_keys = active_keys.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
                let mut current_keys=active_keys.get();
                for i in 0..Self::ACTIVE_KEYS_SIZE {
                    if current_keys[i]==event.key_code()  {
                        current_keys[i]=0;
                    }
                }
                active_keys.set(current_keys);
            });
            document
                .add_event_listener_with_callback("keyup", closure.as_ref()
                .unchecked_ref())
                .unwrap();

            closure.forget();
        }

        // Store a queue of typed keys
        let typed_keys:Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
        {
            let typed_keys = typed_keys.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
                let mut typed_keys = typed_keys.borrow_mut();
                typed_keys.push(event.key_code() as u8);
            });
            document
                .add_event_listener_with_callback("keyup", closure.as_ref()
                .unchecked_ref())
                .unwrap();

            closure.forget();
        }

        // Check if mouse button was clicked
        let mouse_clicked = Rc::new(Cell::new(false));
        {
            let mouse_clicked = mouse_clicked.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_:web_sys::MouseEvent| {
                mouse_clicked.set(true);
            });
            document
                .add_event_listener_with_callback("mouseup", closure.as_ref()
                .unchecked_ref())
                .unwrap();

            closure.forget();
        }

        let mouse_down = Rc::new(Cell::new(false));
        {
            let mouse_down = mouse_down.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_:web_sys::MouseEvent| {
                mouse_down.set(true);
            });
            document
                .add_event_listener_with_callback("mousedown", closure.as_ref()
                .unchecked_ref())
                .unwrap();

            closure.forget();
        }
        {
            let mouse_down = mouse_down.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_:web_sys::MouseEvent| {
                mouse_down.set(false);
            });
            document
                .add_event_listener_with_callback("mouseup", closure.as_ref()
                .unchecked_ref())
                .unwrap();

            closure.forget();
        }



        // Get mouse coordinates
        let mouse_coords:Rc<Cell<(i32, i32)>> = Rc::new(Cell::new((0, 0)));
        {
            let mouse_coords = mouse_coords.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event:web_sys::MouseEvent| {
                let raw_x = event.offset_x();
                let raw_y = event.offset_y();

                let bounding_rect = canvas.get_bounding_client_rect();

                let scale_x = (canvas.width() as f64) / bounding_rect.width();
                let scale_y = (canvas.height() as f64) / bounding_rect.height();

                let x = (raw_x as f64) * scale_x;
                let y = (raw_y as f64) * scale_y;

                mouse_coords.set((x as i32, y as i32));
            });
            document
                .add_event_listener_with_callback("mousemove", closure.as_ref()
                .unchecked_ref())
                .unwrap();

            closure.forget();
        }

        return Input {
            active_keys,
            typed_keys,
            mouse_clicked,
            mouse_down,
            mouse_coords,
            last_mouse_state: false,
        }
    }

    pub fn get_typed_keys(&self) -> Vec<u8> {
        let mut typed_keys = self.typed_keys.borrow_mut();
        let return_typed_keys = typed_keys.clone();
        typed_keys.clear();
        return return_typed_keys;
    }

    pub fn is_down(&self, key:u32) -> bool {
        for current_key in self.active_keys.get() {
            if current_key == key {
                return true;
            }
        }

        return false;
    }

    pub fn mouse_clicked(&self) -> bool {
        let return_mouse_clicked = self.mouse_clicked.get();
        self.mouse_clicked.set(false);
        return return_mouse_clicked;
    }

    // Returns (state_changed, new_state)
    pub fn mouse_state_changed(&mut self) -> (bool, bool) {
        let mouse_down = self.mouse_down.get();

        if mouse_down != self.last_mouse_state {
            self.last_mouse_state = mouse_down;
            return (true, mouse_down);
        }

        return (false, false);
    }

    pub fn mouse_coordinates(&self) -> (i32, i32) {
        return self.mouse_coords.get();
    }
}


