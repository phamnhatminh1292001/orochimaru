#![recursion_limit = "256"]

use ethnum::U256;
use rbtree::RBTree;
use std::marker::PhantomData;
use zkmemory::abstract_machine::{AbstractContext, AbstractInstruction, AbstractMachine};
use zkmemory::base::{Base, UsizeConvertible};
use zkmemory::register_machine::AbstractRegisterMachine;
use zkmemory::stack_machine::AbstractStackMachine;
use zkmemory::state_machine::AbstractStateMachine;

#[derive(Debug)]
pub enum MyInstruction<M, K, V, const S: usize, const T: usize>
where
    K: Base<S>,
    V: Base<T>,
{
    Read(K, V),
    Write(K, V),
    Invalid(PhantomData<M>),
}

#[derive(Debug)]
pub struct MyContext<M, K, V, const S: usize, const T: usize>
where
    K: Base<S>,
    V: Base<T>,
{
    pub(crate) time_log: usize,
    pub(crate) stack_base: K,
    pub(crate) stack_depth: usize,
    pub(crate) stack_ptr: K,
    phantom_m: PhantomData<M>,
    phantom_v: PhantomData<V>,
}

/// RAM Machine
#[derive(Debug)]
pub struct StateMachine<K, V, const S: usize, const T: usize>
where
    K: Base<S>,
    V: Base<T>,
{
    memory: RBTree<K, V>,
    context: MyContext<Self, K, V, S, T>,
    pub(crate) cell_size: K,
}

impl<M, K, V, const S: usize, const T: usize> AbstractContext<M, K, V> for MyContext<M, K, V, S, T>
where
    Self: core::fmt::Debug + Sized,
    K: Base<S>,
    V: Base<T>,
    M: AbstractMachine<K, V>,
{
    fn set_stack_depth(&mut self, stack_depth: usize) {
        todo!()
    }

    fn get_stack_depth(&self) -> usize {
        todo!()
    }

    fn stack_ptr(&self) -> K {
        todo!()
    }

    fn get_time_log(&self) -> usize {
        todo!()
    }

    fn set_time_log(&mut self, time_log: usize) {
        todo!()
    }

    fn apply(&'static mut self, instruction: &<M as AbstractMachine<K, V>>::Instruction) {
        todo!()
    }
}

impl<M, K, V, const S: usize, const T: usize> AbstractInstruction<M, K, V>
    for MyInstruction<M, K, V, S, T>
where
    Self: core::fmt::Debug + Sized,
    K: Base<S>,
    V: Base<T>,
    M: AbstractMachine<K, V>,
{
    fn exec(&self, context: &'static mut <M as AbstractMachine<K, V>>::Context) {
        todo!()
    }
}

impl<K, V, const S: usize, const T: usize> AbstractMachine<K, V> for StateMachine<K, V, S, T>
where
    K: Base<S>,
    V: Base<T>,
{
    type Context = MyContext<Self, K, V, S, T>;
    type Instruction = MyInstruction<Self, K, V, S, T>;
}

impl<K, V, const S: usize, const T: usize> StateMachine<K, V, S, T>
where
    K: Base<S>,
    V: Base<T>,
{
    fn new() -> Self {
        Self {
            memory: RBTree::new(),
            context: MyContext {
                time_log: 0,
                stack_base: K::zero(),
                stack_depth: 0,
                stack_ptr: K::zero(),
                phantom_m: PhantomData,
                phantom_v: PhantomData,
            },
            cell_size: K::from_usize(K::CELL_SIZE),
        }
    }
}

impl<K, V, const S: usize, const T: usize> AbstractStateMachine<K, V> for StateMachine<K, V, S, T>
where
    K: Base<S>,
    V: Base<T>,
    Self: AbstractMachine<K, V>,
{
    fn read(&self, address: K) -> V {
        let remain = address % self.cell_size;
        if remain.is_zero() {
            // Read on a cell
            self.dummy_read(address)
        } else {
            // Get the address of 2 cells
            let (addr_lo, addr_hi) = self.compute_address(address, remain);

            // Get the 2 cells
            let val_lo = self.dummy_read(addr_lo);
            let val_hi = self.dummy_read(addr_hi);
            let cell_size = self.cell_size.to_usize();
            let part_lo = (address - addr_lo).to_usize();
            let part_hi = cell_size - part_lo;
            let mut buf = [0u8; T];

            // Write the value into the buffer
            buf[part_hi..cell_size].copy_from_slice(&val_hi.to_bytes()[0..part_lo]);
            buf[0..part_hi].copy_from_slice(&val_lo.to_bytes()[part_lo..cell_size]);

            V::from_bytes(buf)
        }
    }

    fn write(&mut self, address: K, value: V) {
        let remain = address % self.cell_size;
        if remain.is_zero() {
            // Write on a cell
            self.memory.insert(address, value);
        } else {
            // Get the address of 2 cells
            let (addr_lo, addr_hi) = self.compute_address(address, remain);
            // Calculate memory address and offset
            let cell_size = self.cell_size.to_usize();
            let part_lo = (address - addr_lo).to_usize();
            let part_hi = cell_size - part_lo;

            let val = value.to_bytes();

            // Write the low part of value to the buffer
            let mut buf = self.dummy_read(addr_lo).to_bytes();
            buf[part_lo..cell_size].copy_from_slice(&val[0..part_hi]);
            let val_lo = V::from_bytes(buf);

            // Write the high part of value to the buffer
            let mut buf = self.dummy_read(addr_hi).to_bytes();
            buf[0..part_lo].copy_from_slice(&val[part_hi..cell_size]);
            let val_hi = V::from_bytes(buf);

            self.memory.replace_or_insert(addr_lo, val_lo);
            self.memory.replace_or_insert(addr_hi, val_hi);
        }
    }

    fn dummy_read(&self, address: K) -> V {
        match self.memory.get(&address) {
            Some(r) => r.clone(),
            None => V::zero(),
        }
    }

    fn compute_address(&self, address: K, remain: K) -> (K, K) {
        let base = address - remain;
        (base, base + self.cell_size)
    }

    fn context(&mut self) -> &'_ mut <Self as AbstractMachine<K, V>>::Context {
        todo!()
    }
}

impl<K, V, const S: usize, const T: usize> AbstractRegisterMachine<K, V, S, T>
    for StateMachine<K, V, S, T>
where
    K: Base<S>,
    V: Base<T>,
    Self: AbstractStateMachine<K, V>,
{
}

impl<K, V, const S: usize, const T: usize> AbstractStackMachine<K, V> for StateMachine<K, V, S, T>
where
    K: Base<S>,
    V: Base<T>,
    Self: AbstractStateMachine<K, V>,
{
}

fn main() {
    let mut a = StateMachine::<U256, U256, 32, 32>::new();
    a.write(U256::from_usize(0), U256::from_usize(123));

    /*
    // Test the state machine of Uint256 values
    let mut sm = StateMachine256::new(DefaultConfig::default());

    let base_address: usize = sm.base_address().to_usize();
    sm.write(
        U256::from_usize(base_address),
        U256::from_be_bytes([1u8; 32]),
    )
    .unwrap();
    sm.write(
        U256::from_usize(base_address + 32),
        U256::from_be_bytes([2u8; 32]),
    )
    .unwrap();

    sm.write(
        U256::from_usize(base_address + 6),
        U256::from_be_bytes([3u8; 32]),
    )
    .unwrap();

    println!("{:?}", sm.read(U256::from_usize(base_address + 7)).unwrap());

    println!("{:?}", sm.read(U256::from_usize(base_address + 0)).unwrap());

    println!(
        "{:?}",
        sm.read(U256::from_usize(base_address + 32)).unwrap()
    );

    sm.push(U256::from_usize(123)).unwrap();

    sm.pop().unwrap();

    let r0 = sm.register(0).unwrap();
    let r1 = sm.register(1).unwrap();

    sm.set(r1, U256::from_be_bytes([9u8; 32])).unwrap();
    sm.mov(r0, r1).unwrap();

    // Check the memory trace
    println!("{:#064x?}", sm);

    let trace = sm.trace();

    println!("{:#064x?}", trace); */
}
