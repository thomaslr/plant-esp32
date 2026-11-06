//! Pin Diagnostic: Toggles GPIO 5, GPIO 20, and GPIO 21
//! to identify which exact GPIO is physically routed to TB2 Pin 2!

use std::thread::sleep;
use std::time::Duration;

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::{
    gpio_mode_t_GPIO_MODE_OUTPUT, gpio_reset_pin, gpio_set_direction, gpio_set_level, link_patches,
};

fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    println!("\n========================================================");
    println!(" 🔍 PIN IDENTIFIER TEST (GPIO 5 vs GPIO 20 vs GPIO 21)");
    println!("========================================================");
    println!(" We will toggle each candidate pin one by one.");
    println!(" Watch your voltmeter on TB2 Pin 2 to see which phase toggles it!");
    println!("--------------------------------------------------------\n");

    let pins = [5, 20, 21];
    for &p in &pins {
        unsafe {
            gpio_reset_pin(p);
            gpio_set_direction(p, gpio_mode_t_GPIO_MODE_OUTPUT);
            gpio_set_level(p, 0); // start all LOW
        }
    }

    loop {
        // --- PHASE A: Drive GPIO 5 HIGH (others LOW) for 3s ---
        println!(">>> [TEST 1] ONLY GPIO 5 is HIGH (3.3V). GPIO 20 & 21 are 0V.");
        unsafe {
            gpio_set_level(20, 0);
            gpio_set_level(21, 0);
            gpio_set_level(5, 1);
        }
        sleep(Duration::from_secs(3));

        // --- PHASE B: Drive GPIO 20 HIGH (others LOW) for 3s ---
        println!(">>> [TEST 2] ONLY GPIO 20 is HIGH (3.3V). GPIO 5 & 21 are 0V.");
        unsafe {
            gpio_set_level(5, 0);
            gpio_set_level(21, 0);
            gpio_set_level(20, 1);
        }
        sleep(Duration::from_secs(3));

        // --- PHASE C: Drive GPIO 21 HIGH (others LOW) for 3s ---
        println!(">>> [TEST 3] ONLY GPIO 21 is HIGH (3.3V). GPIO 5 & 20 are 0V.");
        unsafe {
            gpio_set_level(5, 0);
            gpio_set_level(20, 0);
            gpio_set_level(21, 1);
        }
        sleep(Duration::from_secs(3));

        // --- ALL OFF for 2s ---
        println!(">>> [ALL OFF] GPIO 5, 20, 21 are all 0.0V.");
        unsafe {
            gpio_set_level(5, 0);
            gpio_set_level(20, 0);
            gpio_set_level(21, 0);
        }
        sleep(Duration::from_secs(2));
    }
}
