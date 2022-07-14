#![feature(generators, generator_trait)]

use desim::EndCondition;
use desim_benchmark::test_simulation;

fn main() {
    let (simulation, state, count_key) = test_simulation();
    println!("Running the simulation");    
    let simulation = simulation.run(EndCondition::Time(1002.0));
    
    let mut state = state.take();
    let count = state.remove(count_key).unwrap();
    
    println!("Final Count = {}", count);
    println!("Final Time = {}s", simulation.time());
}
