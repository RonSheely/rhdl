use crate::{
    clock::clock, clock_reset, timed_sample, types::reset::reset, Clock, ClockReset, Digital,
    TimedSample,
};

use super::ResetOrData;

#[derive(Clone)]
enum State {
    Init,
    Hold,
    ClockLow,
    ClockHigh,
    Done,
}

pub struct ClockPosEdge<I, S>
where
    S: Digital,
{
    input: I,
    sample: ResetOrData<S>,
    state: State,
    time: u64,
    next_time: u64,
    period: u64,
}

impl<I, S> ClockPosEdge<I, S>
where
    S: Digital,
{
    fn this_sample(&self, clock: Clock) -> TimedSample<(ClockReset, S)> {
        match self.sample {
            ResetOrData::Data(x) => timed_sample(self.time, (clock_reset(clock, reset(false)), x)),
            ResetOrData::Reset => {
                timed_sample(self.time, (clock_reset(clock, reset(true)), S::init()))
            }
        }
    }
}

impl<I, S> Clone for ClockPosEdge<I, S>
where
    I: Clone,
    S: Clone + Digital,
{
    fn clone(&self) -> Self {
        ClockPosEdge {
            input: self.input.clone(),
            sample: self.sample,
            state: self.state.clone(),
            time: self.time,
            next_time: self.next_time,
            period: self.period,
        }
    }
}

//
// The waveform
//
//  data    ----------------*---------------------------*
//             x1           |      x2                   |
//          ----------------*---------------------------*
//
//                     *----------------*               *-----*
//  clk                |                |               |
//          *----------*                *---------------*
//
//  state  init        ^ hold ^ high    ^      low      ^ hold
//
//
//   clk    l          h      h         l               h
//    x     x1         x1     x2        x2              x2
//

impl<I, S> Iterator for ClockPosEdge<I, S>
where
    I: Iterator<Item = ResetOrData<S>>,
    S: Digital,
{
    type Item = TimedSample<(ClockReset, S)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            State::Init => {
                if let Some(data) = self.input.next() {
                    self.sample = data;
                    self.state = State::Hold;
                    self.next_time = self.time + self.period / 2;
                    Some(self.this_sample(clock(false)))
                } else {
                    self.state = State::Done;
                    None
                }
            }
            State::ClockLow => {
                self.state = State::Hold;
                self.time = self.next_time;
                self.next_time = self.time + 1;
                Some(self.this_sample(clock(true)))
            }
            State::Hold => {
                if let Some(data) = self.input.next() {
                    self.sample = data;
                    self.state = State::ClockHigh;
                    self.next_time = self.time + self.period / 2 - 1;
                    Some(self.this_sample(clock(true)))
                } else {
                    self.state = State::Done;
                    None
                }
            }
            State::ClockHigh => {
                self.state = State::ClockLow;
                self.time = self.next_time;
                self.next_time = self.time + self.period / 2;
                Some(self.this_sample(clock(false)))
            }
            State::Done => None,
        }
    }
}

pub fn clock_pos_edge<I, S>(input: I, period: u64) -> ClockPosEdge<I, S>
where
    I: Iterator<Item = ResetOrData<S>>,
    S: Digital,
{
    ClockPosEdge {
        input,
        sample: ResetOrData::Reset,
        state: State::Init,
        time: 0,
        next_time: 0,
        period,
    }
}

pub trait ClockPosEdgeExt<Q>: Iterator
where
    Q: Digital,
{
    fn clock_pos_edge(self, period: u64) -> impl Iterator<Item = TimedSample<(ClockReset, Q)>>;
}

impl<I, Q> ClockPosEdgeExt<Q> for I
where
    I: Iterator<Item = ResetOrData<Q>>,
    Q: Digital,
{
    fn clock_pos_edge(self, period: u64) -> impl Iterator<Item = TimedSample<(ClockReset, Q)>> {
        clock_pos_edge(self, period)
    }
}
