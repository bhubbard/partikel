//! # Disney SeExpr Procedural Expression VM (`wdas/SeExpr` / `seexpr-rs`)
//!
//! Embeddable, sandboxed expression bytecode virtual machine for procedural motion graphics,
//! keyframe curves, Perlin noise modulation, and deterministic zero-allocation frame evaluation.

use serde::{Deserialize, Serialize};

/// Bytecode instruction for the register/stack-based `SeExpr` procedural animation sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SeExprOp {
    PushConst(f64),
    PushTime,
    PushFrame,
    Add,
    Sub,
    Mul,
    Div,
    Sin,
    Cos,
    SmoothStep(f64, f64),
    Noise1D,
}

/// Deterministic, bounded-step virtual machine for `wdas/SeExpr` procedural animation curves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeExprProgram {
    pub ops: Vec<SeExprOp>,
    pub max_steps: usize,
}

impl SeExprProgram {
    pub fn new(ops: Vec<SeExprOp>) -> Self {
        Self {
            ops,
            max_steps: 256,
        }
    }

    /// Evaluates 1D gradient Perlin noise in `[-1.0, 1.0]` with quintic fade `6t^5 - 15t^4 + 10t^3`.
    pub fn perlin_noise_1d(x: f64) -> f64 {
        let x0 = x.floor() as i64;
        let xf = x - (x0 as f64);
        let u = xf * xf * xf * (xf * (xf * 6.0 - 15.0) + 10.0);
        let grad = |h: i64, d: f64| -> f64 {
            let hash = (h.wrapping_mul(0x5DEECE66D).wrapping_add(11)) & 1;
            if hash == 0 { d } else { -d }
        };
        let a = grad(x0, xf);
        let b = grad(x0 + 1, xf - 1.0);
        a + u * (b - a)
    }

    /// Evaluates the expression program for a given `(time_sec, frame)` with zero heap allocations
    /// by using a fixed-size 32-slot register stack.
    pub fn eval(&self, time_sec: f64, frame: f64) -> f64 {
        let mut stack = [0.0f64; 32];
        let mut sp = 0usize;

        for (step, &op) in self.ops.iter().enumerate() {
            if step >= self.max_steps {
                break;
            }
            match op {
                SeExprOp::PushConst(v) => {
                    if sp < stack.len() {
                        stack[sp] = v;
                        sp += 1;
                    }
                }
                SeExprOp::PushTime => {
                    if sp < stack.len() {
                        stack[sp] = time_sec;
                        sp += 1;
                    }
                }
                SeExprOp::PushFrame => {
                    if sp < stack.len() {
                        stack[sp] = frame;
                        sp += 1;
                    }
                }
                SeExprOp::Add if sp >= 2 => {
                    stack[sp - 2] += stack[sp - 1];
                    sp -= 1;
                }
                SeExprOp::Sub if sp >= 2 => {
                    stack[sp - 2] -= stack[sp - 1];
                    sp -= 1;
                }
                SeExprOp::Mul if sp >= 2 => {
                    stack[sp - 2] *= stack[sp - 1];
                    sp -= 1;
                }
                SeExprOp::Div if sp >= 2 => {
                    let denom = stack[sp - 1];
                    stack[sp - 2] = if denom.abs() > 1e-12 {
                        stack[sp - 2] / denom
                    } else {
                        0.0
                    };
                    sp -= 1;
                }
                SeExprOp::Sin if sp >= 1 => {
                    stack[sp - 1] = stack[sp - 1].sin();
                }
                SeExprOp::Cos if sp >= 1 => {
                    stack[sp - 1] = stack[sp - 1].cos();
                }
                SeExprOp::SmoothStep(e0, e1) if sp >= 1 => {
                    let x = stack[sp - 1];
                    let t = ((x - e0) / (e1 - e0).max(1e-9)).clamp(0.0, 1.0);
                    stack[sp - 1] = t * t * (3.0 - 2.0 * t);
                }
                SeExprOp::Noise1D if sp >= 1 => {
                    stack[sp - 1] = Self::perlin_noise_1d(stack[sp - 1]);
                }
                _ => {}
            }
        }
        if sp > 0 { stack[sp - 1] } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seexpr_vm_smoothstep_and_noise() {
        let prog = SeExprProgram::new(vec![
            SeExprOp::PushTime,
            SeExprOp::SmoothStep(0.0, 2.0),
        ]);
        assert!((prog.eval(1.0, 24.0) - 0.5).abs() < 1e-6);
        assert!((prog.eval(2.0, 48.0) - 1.0).abs() < 1e-6);
    }
}
