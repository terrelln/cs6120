use cs6120::bril;
use cs6120::lvn;

fn main() {
    let program = bril::load_program();
    let program = lvn::lvn(&program);
    bril::output_program(&program);
}