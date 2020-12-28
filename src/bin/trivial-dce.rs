use cs6120::bril;
use cs6120::dce;

fn main() {
    let program = bril::load_program();
    let program = dce::trivial_dce(&program);
    bril::output_program(&program);
}