#![feature(generators, generator_trait)]

mod state;

use std::{rc::Rc, cell::Cell};

use desim::{Effect, SimGen, Simulation, SimContext};
use state::StateKey;

use crate::state::State;

#[derive(PartialEq, Clone, Copy)]
enum Passivated {
    True,
    False,
    Warned,
}

pub fn simulation(limit: f64) {
    let simulation = set_simulation();
    simulation.run(desim::EndCondition::Time(limit));
}

fn set_simulation() -> Simulation<Effect> {
    let mut simulation = Simulation::new();
    
    let shared_state = Rc::new(Cell::new(State::default()));
    
    let mut state = shared_state.take();
    let count_key = state.insert(0);
    
    let producer_key = state.insert(None);
    let consumer_key = state.insert(None);
    let passivated_key = state.insert([Passivated::False; 2]);
    
    let producer = simulation.create_process(producer(Rc::clone(&shared_state), passivated_key, consumer_key, count_key));
    let consumer = simulation.create_process(consumer(Rc::clone(&shared_state), passivated_key, producer_key, count_key));
    
    *state.get_mut(producer_key).unwrap() = Some(producer); 
    *state.get_mut(consumer_key).unwrap() = Some(consumer); 
        
    simulation.schedule_event(0.0, producer, Effect::TimeOut(0.0));
    simulation.schedule_event(0.0, consumer, Effect::TimeOut(0.0));
    
    shared_state.set(state);
    
    simulation
}

fn producer(shared_state: Rc<Cell<State>>, passivated_key: StateKey<[Passivated; 2]>, consumer_key: StateKey<Option<usize>>, count_key: StateKey<usize>) -> Box<SimGen<Effect>> {
    Box::new(move |x: SimContext<Effect>| {
        let shared_state = shared_state;
        let produce_amount = 1;
        let thresh_hold = 15;
        let interval = 1.0;

        let mut state = shared_state.take();
        let consumer_key = state.remove(consumer_key).flatten().unwrap();
        shared_state.set(state);
        
        loop {
            let mut state = shared_state.take();
            let passivated_list = state.get_mut(passivated_key).unwrap();
            passivated_list[0] = Passivated::False;
            
            if passivated_list[1] == Passivated::True {
                passivated_list[1] = Passivated::Warned;
                passivated_list[0] = Passivated::True;
                shared_state.set(state);

                // Diferente a mi libreria, esto no volvera a despertar a producer.
                // Efectivamente producer hace Passivate aqui;
                // println!("PRODUCER ACTIVATE CONSUMER and PASSIVATE");
                yield Effect::Event { time: 0.0, process: consumer_key } ;
                // println!("PRODUCER awakes");
                let mut state = shared_state.take();
                let passivated_list = state.get_mut(passivated_key).unwrap();
                passivated_list[0] = Passivated::False;
                shared_state.set(state);
            } else {
                shared_state.set(state);
            }            

            let mut state = shared_state.take();            
            let count = state.get_mut(count_key).unwrap();

            if *count < thresh_hold {
                *count += produce_amount;
                // println!("PRODUCED - Before: {} | After: {} | At: {}s", *count - produce_amount, *count, x.time());
                shared_state.set(state);
                // println!("PRODUCER make a hold");
                yield Effect::TimeOut(interval);
            } else {
                let passivated_list = state.get_mut(passivated_key).unwrap();
                passivated_list[0] = Passivated::True;
                if passivated_list[1] == Passivated::True {
                    shared_state.set(state);
                //     println!("PRODUCER PASSIVATE and ACTIVATE CONSUMER");
                    yield Effect::Event { time: 0.0 , process: consumer_key };
                } else {
                    shared_state.set(state);
                //     println!("PRODUCER PASSIVATE");
                    yield Effect::Wait;
                }
            }
        }
    })
}

fn consumer(shared_state: Rc<Cell<State>>, passivated_key: StateKey<[Passivated; 2]>, producer_key: StateKey<Option<usize>>, count_key: StateKey<usize>) -> Box<SimGen<Effect>> {
    Box::new(move |x: SimContext<Effect>| {        
        let shared_state = shared_state;
        let consume_amount = 8;
        let interval = 8.0;

        let mut state = shared_state.take();
        let producer_key = state.remove(producer_key).flatten().unwrap();
        shared_state.set(state);
        
        loop {
            let mut state = shared_state.take();
            let passivated_list = state.get_mut(passivated_key).unwrap();
            passivated_list[1] = Passivated::False;
            
            if passivated_list[0] == Passivated::True {
                passivated_list[0] = Passivated::Warned;
                passivated_list[1] = Passivated::True;
                shared_state.set(state);
                // println!("CONSUMER ACTIVATE PRODUCER and PASSIVATE");
                yield Effect::Event { time: 0.0, process: producer_key };
                // println!("CONSUMER awakes");
                let mut state = shared_state.take();
                let passivated_list = state.get_mut(passivated_key).unwrap();
                passivated_list[1] = Passivated::False;
                shared_state.set(state);
            } else {
                shared_state.set(state);
            }

            let mut state = shared_state.take();
            let count = state.get_mut(count_key).unwrap();
            if *count >= consume_amount {
                *count -= consume_amount;
                // println!("CONSUMED - Before: {} | After: {} | At: {:?}", *count + consume_amount, *count, x.time());
                shared_state.set(state);
                // println!("CONSUMER make a hold");
                yield Effect::TimeOut(interval);
            } else {
                let passivated_list = state.get_mut(passivated_key).unwrap();
                passivated_list[1] = Passivated::True;
                if passivated_list[0] == Passivated::True {
                    shared_state.set(state);
                //     println!("CONSUMER PASSIVATE and ACTIVATE PRODUCER");
                    yield Effect::Event { time: 0.0, process: producer_key };
                } else {
                    shared_state.set(state);
                //     println!("CONSUMER PASSIVATE");
                    yield Effect::Wait;
                }
            }
        }
    })
}


pub fn test_simulation() -> (Simulation<Effect>, Rc<Cell<State>>, StateKey<usize>) {
    let mut simulation = Simulation::new();
    
    let shared_state = Rc::new(Cell::new(State::default()));
    
    let mut state = shared_state.take();
    let count_key = state.insert(0);
    
    let producer_key = state.insert(None);
    let consumer_key = state.insert(None);
    let passivated_key = state.insert([Passivated::False; 2]);
    
    let producer = simulation.create_process(producer(Rc::clone(&shared_state), passivated_key, consumer_key, count_key));
    let consumer = simulation.create_process(consumer(Rc::clone(&shared_state), passivated_key, producer_key, count_key));
    
    *state.get_mut(producer_key).unwrap() = Some(producer); 
    *state.get_mut(consumer_key).unwrap() = Some(consumer); 
        
    simulation.schedule_event(0.0, producer, Effect::TimeOut(0.0));
    simulation.schedule_event(0.0, consumer, Effect::TimeOut(0.0));
    
    shared_state.set(state);
    
    (simulation, shared_state, count_key)
}