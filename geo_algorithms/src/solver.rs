use geo_core::ToleranceContext;

#[derive(Debug, Clone)]
pub struct ConvergenceInfo {
    pub iterations: usize,
    pub residual: f64,
    pub converged: bool,
    pub final_error: f64,
}

pub struct NewtonSolver {
    tolerance: ToleranceContext,
    max_iterations: usize,
}

impl NewtonSolver {
    pub fn new(tolerance: ToleranceContext) -> Self {
        Self { tolerance, max_iterations: 100 }
    }

    pub fn solve_1d<F, G>(&self, function: F, derivative: G, initial_guess: f64)
        -> Result<(f64, ConvergenceInfo), String>
    where F: Fn(f64)->f64, G: Fn(f64)->f64
    {
        let mut x = initial_guess;
        let mut info = ConvergenceInfo { iterations:0, residual:f64::INFINITY, converged:false, final_error:f64::INFINITY };
        for i in 0..self.max_iterations {
            let f_val = function(x);
            let df_val = derivative(x);
            if df_val.abs() < self.tolerance.parametric { return Err("Derivative too small, cannot continue".into()); }
            let delta = f_val / df_val;
            x -= delta;
            info.iterations = i+1;
            info.residual = f_val.abs();
            info.final_error = delta.abs();
            if info.residual < self.tolerance.parametric && info.final_error < self.tolerance.parametric { info.converged = true; break; }
        }
        Ok((x, info))
    }

    pub fn solve_2d<F>(&self, system: F, initial_guess: (f64,f64))
        -> Result<((f64,f64), ConvergenceInfo), String>
    where F: Fn(f64,f64)->(f64,f64,[[f64;2];2])
    {
        let (mut x, mut y) = initial_guess;
        let mut info = ConvergenceInfo { iterations:0, residual:f64::INFINITY, converged:false, final_error:f64::INFINITY };
        for i in 0..self.max_iterations {
            let (f1,f2,j) = system(x,y);
            let det = j[0][0]*j[1][1] - j[0][1]*j[1][0];
            if det.abs() < self.tolerance.parametric { return Err("Singular Jacobian".into()); }
            let inv_det = 1.0/det;
            let dx = inv_det * (j[1][1]*f1 - j[0][1]*f2);
            let dy = inv_det * (-j[1][0]*f1 + j[0][0]*f2);
            x -= dx; y -= dy;
            info.iterations = i+1;
            info.residual = (f1*f1 + f2*f2).sqrt();
            info.final_error = (dx*dx + dy*dy).sqrt();
            if info.residual < self.tolerance.parametric && info.final_error < self.tolerance.parametric { info.converged = true; break; }
        }
        Ok(((x,y), info))
    }
}
