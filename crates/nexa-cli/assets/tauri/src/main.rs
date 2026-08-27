// Evita una consola adicional en Windows en modo release — no la quites.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    {{NAME_UNDERSCORE}}_lib::run();
}
