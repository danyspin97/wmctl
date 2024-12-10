use std::fmt::Display;

use eyre::Result;
use log::warn;
use smithay_client_toolkit::{
    delegate_output, delegate_registry,
    output::{OutputHandler, OutputInfo, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};
use wayland_client::{
    globals::registry_queue_init, protocol::wl_output, Connection, EventQueue, QueueHandle,
};

/// Application data.
///
/// This type is where the delegates for some parts of the protocol and any application specific data will
/// live.
pub struct WaylandClient {
    registry_state: RegistryState,
    output_state: OutputState,
    output_changed: bool,
}

impl WaylandClient {
    pub fn new() -> Result<(Self, EventQueue<WaylandClient>)> {
        // Try to connect to the Wayland server.
        let conn = Connection::connect_to_env()?;

        // Now create an event queue and a handle to the queue so we can create objects.
        let (globals, mut event_queue) = registry_queue_init(&conn)?;
        let qh = event_queue.handle();

        // Initialize the registry handling so other parts of Smithay's client toolkit may bind
        // globals.
        let registry_state = RegistryState::new(&globals);

        // Initialize the delegate we will use for outputs.
        let output_delegate = OutputState::new(&globals, &qh);

        // Set up application state.
        //
        // This is where you will store your delegates and any data you wish to access/mutate while the
        // application is running.
        let mut wayland_client = WaylandClient {
            registry_state,
            output_state: output_delegate,
            output_changed: false,
        };

        // `OutputState::new()` binds the output globals found in `registry_queue_init()`.
        //
        // After the globals are bound, we need to dispatch again so that events may be sent to the newly
        // created objects.
        event_queue.roundtrip(&mut wayland_client)?;

        Ok((wayland_client, event_queue))
    }

    /// List all outputs. for the connected Wayland server.
    pub fn list_outputs(&self, short: bool, json: bool) {
        let outputs = self.output_state.outputs().filter_map(|output| {
            if let Some(info) = &self.output_state.info(&output) {
                Some(DisplayOutput::new(info))
            } else {
                warn!("No output info found for {:?}", output);
                None
            }
        });
        if json {
            println!(
                "{}",
                serde_json::to_string(&outputs.collect::<Vec<DisplayOutput>>()).unwrap()
            );
        } else if short {
            for output in outputs {
                println!("{}", output.model);
            }
        } else {
            for output in outputs {
                println!("{}", output);
            }
        }
    }

    pub fn watch_for_output_changes(&mut self, mut event_queue: EventQueue<Self>) {
        // Reset the status here
        self.output_changed = false;
        loop {
            // Dispatch events until the new_output or output_destroyed gets called
            if self.output_changed {
                break;
            }
            event_queue.blocking_dispatch(self).unwrap();
        }
    }
}

// In order to use OutputDelegate, we must implement this trait to indicate when something has happened to an
// output and to provide an instance of the output state to the delegate when dispatching events.
impl OutputHandler for WaylandClient {
    // First we need to provide a way to access the delegate.
    //
    // This is needed because delegate implementations for handling events use the application data type in
    // their function signatures. This allows the implementation to access an instance of the type.
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    // Then there exist these functions that indicate the lifecycle of an output.
    // These will be called as appropriate by the delegate implementation.

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        self.output_changed = true;
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        self.output_changed = true;
    }
}

// Now we need to say we are delegating the responsibility of output related events for our application data
// type to the requisite delegate.
delegate_output!(WaylandClient);

// In order for our delegate to know of the existence of globals, we need to implement registry
// handling for the program. This trait will forward events to the RegistryHandler trait
// implementations.
delegate_registry!(WaylandClient);

// In order for delegate_registry to work, our application data type needs to provide a way for the
// implementation to access the registry state.
//
// We also need to indicate which delegates will get told about globals being created. We specify
// the types of the delegates inside the array.
impl ProvidesRegistryState for WaylandClient {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers! {
        // Here we specify that OutputState needs to receive events regarding the creation and destruction of
        // globals.
        OutputState,
    }
}

#[derive(serde::Serialize)]
struct DisplayOutput {
    model: String,
    name: Option<String>,
    description: Option<String>,
    make: String,
    location: (i32, i32),
    subpixel: String,
    physical_size: (i32, i32),
    logical_position: Option<(i32, i32)>,
    logical_size: Option<(i32, i32)>,
    modes: Vec<String>,
}

fn subpixel_to_string(subpixel: wl_output::Subpixel) -> String {
    match subpixel {
        wl_output::Subpixel::None => "None".to_string(),
        wl_output::Subpixel::HorizontalRgb => "Horizontal RGB".to_string(),
        wl_output::Subpixel::HorizontalBgr => "Horizontal BGR".to_string(),
        wl_output::Subpixel::VerticalRgb => "Vertical RGB".to_string(),
        wl_output::Subpixel::VerticalBgr => "Vertical BGR".to_string(),
        _ => "Unknown".to_string(),
    }
}

impl DisplayOutput {
    pub fn new(info: &OutputInfo) -> Self {
        let model = info.model.clone();
        let name = info.name.clone();
        let description = info.description.clone();
        let make = info.make.clone();
        let location = info.location;
        // The subpixel enum is not serializable, so we convert it to a string.
        let subpixel = subpixel_to_string(info.subpixel);
        let physical_size = info.physical_size;
        let logical_position = info.logical_position;
        let logical_size = info.logical_size;
        let modes = info.modes.iter().map(|m| m.to_string()).collect();
        DisplayOutput {
            model,
            name,
            description,
            make,
            location,
            subpixel,
            physical_size,
            logical_position,
            logical_size,
            modes,
        }
    }
}

/// Prints some [`DisplayOutput`].
impl Display for DisplayOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.model)?;

        if let Some(name) = self.name.as_ref() {
            writeln!(f, "\tname: {name}")?;
        }

        if let Some(description) = self.description.as_ref() {
            writeln!(f, "\tdescription: {description}")?;
        }

        writeln!(f, "\tmake: {}", self.make)?;
        writeln!(f, "\tx: {}, y: {}", self.location.0, self.location.1)?;
        writeln!(f, "\tsubpixel: {:?}", self.subpixel)?;
        writeln!(
            f,
            "\tphysical_size: {}×{}mm",
            self.physical_size.0, self.physical_size.1
        )?;
        if let Some((x, y)) = self.logical_position.as_ref() {
            writeln!(f, "\tlogical x: {x}, y: {y}")?;
        }
        if let Some((width, height)) = self.logical_size.as_ref() {
            writeln!(f, "\tlogical width: {width}, height: {height}")?;
        }
        writeln!(f, "\tmodes:")?;

        for mode in &self.modes {
            writeln!(f, "\t\t{mode}")?;
        }

        Ok(())
    }
}
