extern crate rand;

use crate::phv::Phv;
use crate::alu::ALU;

pub struct PipelineStage {
   pub stateless_alus: Vec<ALU>,
   pub stateful_alus: Vec<ALU>,
   pub salu_configs: Vec<i32>,
   pub output_mux_globals: Vec<i32>,
}

impl PipelineStage {
  pub fn new () -> Self {
    PipelineStage { stateless_alus: Vec::new(),
                    stateful_alus: Vec::new(), 
                    salu_configs: vec![0], 
                    output_mux_globals: vec![0],
    }
  }
  pub fn with_alus (t_stateless_alus: Vec <ALU>, 
                    t_stateful_alus: Vec<ALU>, 
                    t_salu_configs: Vec<i32>) -> Self{
    PipelineStage { stateless_alus: t_stateless_alus,
                    stateful_alus: t_stateful_alus,
                    salu_configs: t_salu_configs,
                    output_mux_globals: vec![0],
      }
  }

  // Iterates through all alus stored and calls their 
  // underlying function on the incoming Phv in 
  // random order. Pass the mutated phv containers to their respective muxes.
  pub fn tick(&mut self, 
              t_initial_phv : Phv <i32>,
              t_input_phv : Phv<i32>) -> (Phv<i32>,Phv<i32>){ 

      let mut input_phv : Phv <i32> = t_input_phv.clone();
      let mut initial_phv : Phv <i32> = t_initial_phv.clone();
      if input_phv.is_bubble() {
        (Phv::new(), Phv::new())
      }
      else{
        let mut output_phv : Phv<i32> = 
            Phv { bubble : false, 
                  phv_containers: Vec::new(),
                  state : Vec::new()};

        let mut state_for_output_mux = Vec::new();
        // List of new state variables for output mux
        let mut new_state = Vec::new();
        // Need old state variables first to put them
        // into output muxes later
        let mut alu_count: usize = 0;

        for alu in self.stateful_alus.iter_mut () {
          if self.salu_configs[alu_count] == 1 {
              let t_input_state = input_phv.get_state().clone();
              input_phv.set_state(t_input_state);

              let t_initial_state = initial_phv.get_state();
              initial_phv.set_state(t_initial_state);

            alu.set_state_variables 
                (input_phv.get_state()[alu_count].clone());
          }
          alu.send_packets_to_input_muxes(input_phv.clone());
          let mut packet_fields = alu.input_mux_output();
          let state_result = alu.run (&mut packet_fields);
          let mut old_state_result = state_result.0;

          let new_state_result = state_result.1;

          if self.output_mux_globals[alu_count] == 1 {
            state_for_output_mux.append(&mut old_state_result);
          }
          else {
            state_for_output_mux.append(&mut new_state_result.clone());
          }
          new_state.push (new_state_result);
          alu.reset_state_variables();
          alu_count+=1;
        }

        // Gets return values from the ALUs and inserts
        // them into output muxes along with old state vars
        for alu in self.stateless_alus.iter_mut() {
        
          //PHV is passed to it's corresponding input mux, and
          //a single container is outputted. Container is put
          //into a vector and passed to alu
          alu.send_packets_to_input_muxes(input_phv.clone());
          let packet_fields = alu.input_mux_output();
          //After being passed to alu, value is sent to an
          //output mux and put into a PHV

          let result : i32 =  alu.run(&packet_fields).0[0];
          // State variables and returned value from stateless ALU
          let mut output_mux_fields : Vec <i32> = state_for_output_mux.clone();

          output_mux_fields.push (result);

          alu.send_packets_to_output_mux(&output_mux_fields);
          output_phv.add_container_to_phv(alu.output_mux.output());
        }
 
        // Update output_phv state variables
        // output_state is the result of the state variables after the
        // stage's execution. 
        let mut output_state = Vec::new();
        for i in 0..self.salu_configs.len() {
          if self.salu_configs[i] == 1 {
            output_state.push (new_state[i].clone());
            // Write to state variables for next PHV
          }
          else {
            output_state.push (input_phv.get_state()[i].clone());
          }
        }
        output_phv.set_state (output_state);
        (initial_phv, output_phv)
      }
    }
}

