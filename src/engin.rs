mod pre_processor;
mod rc_processor;
mod core_processor;
mod trading_session;
mod exe_processor;

use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};

use crate::messages::*;
use crate::types::ExeSender;

use self::exe_processor::{ExeProcessor};
use self::pre_processor::PreProcessor;
use self::rc_processor::RcProcessor;
use self::core_processor::CoreProcessor;

pub struct Engin {
    pub engin_tx : Sender<Option<PreProcessorTask>>,
    pre : Option<JoinHandle<()>>,
    rc : Option<JoinHandle<()>>,
    core : Option<JoinHandle<()>>,
    exe : Option<JoinHandle<ExeSender>>,
}

impl Engin {
    pub fn new(mut sender : ExeSender) -> Engin {
        let (engin_tx, pre_rx) = channel();
        let (pre_tx, rc_rx) = channel();
        let (rc_tx, core_rx) = channel();
        let (core_tx, exe_rx) = channel();

        //let mut tmp_sender = sender;

        Engin {
            engin_tx : engin_tx,
            pre : Some(thread::spawn(move || {
                let mut worker = PreProcessor::new();
                loop {
                    let task = pre_rx.recv().unwrap();
                    if let None = task {
                        pre_tx.send(None).unwrap();
                        break;
                    }
                    //println!("pre recieved one!");
                    pre_tx.send(Some(worker.process(task.unwrap()))).unwrap();
                }
            })),

            rc : Some(thread::spawn(move || {
                let mut worker = RcProcessor::new();
                loop {
                    let task = rc_rx.recv().unwrap();
                    if let None = task {
                        rc_tx.send(None).unwrap();
                        break;
                    }
                    //println!("rc recieved one!");
                    rc_tx.send(Some(worker.process(task.unwrap()))).unwrap();
                }
            })),

            core : Some(thread::spawn(move || {
                let mut worker = CoreProcessor::new();
                let exe_fn = |task : ExecutionTask| {core_tx.send(Some(task)).unwrap();};
                loop {
                    let task = core_rx.recv().unwrap();
                    if let None = task {
                        core_tx.send(None).unwrap();
                        break;
                    }


                    //println!("core recieved one!");
                    worker.process(task.unwrap(), exe_fn);

                }
            })),

            exe : Some(thread::spawn(move || {
                let mut worker = ExeProcessor::new();
                loop {
                    let task = exe_rx.recv().unwrap();
                    if let None = task {
                        break;
                    }
                    worker.process(task.unwrap(), &mut sender);
                }
                sender
            }))
        }
    }
    
    pub fn process(&self, task : PreProcessorTask) {
        self.engin_tx.send(Some(task)).unwrap();
    }

    pub fn close(&mut self) -> ExeSender{
        self.engin_tx.send(None).unwrap();
        self.pre.take().unwrap().join().unwrap();
        self.rc.take().unwrap().join().unwrap();
        self.core.take().unwrap().join().unwrap();
        self.exe.take().unwrap().join().unwrap()
    }

}