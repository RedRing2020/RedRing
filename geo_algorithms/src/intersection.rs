use geo_core::{Point2D, ToleranceContext};
use crate::{NewtonSolver};
use crate::sampling::IntersectionCandidate;

pub struct CurveIntersection {
    tolerance: ToleranceContext,
    newton_solver: NewtonSolver,
}

impl CurveIntersection {
    pub fn new(tolerance: ToleranceContext) -> Self {
        let newton_solver = NewtonSolver::new(tolerance.clone());
        Self { tolerance, newton_solver }
    }

    pub fn find_intersections<F1,F2>(&self, curve1: F1, curve2: F2, t1_range: (f64,f64), t2_range:(f64,f64)) -> Vec<IntersectionCandidate>
    where F1: Fn(f64)->Point2D + Clone, F2: Fn(f64)->Point2D + Clone {
        let mut candidates = Vec::new();
        let grid_size = 20;
        let step1 = (t1_range.1 - t1_range.0)/grid_size as f64;
        let step2 = (t2_range.1 - t2_range.0)/grid_size as f64;
        for i in 0..grid_size { for j in 0..grid_size { let t1 = t1_range.0 + i as f64 * step1; let t2 = t2_range.0 + j as f64 * step2; let p1 = curve1(t1); let p2 = curve2(t2); let distance = p1.distance_to(&p2).value(); if distance < self.tolerance.linear*10.0 { if let Some(r) = self.refine_intersection(&curve1,&curve2,t1,t2) { candidates.push(r); } } } }
        self.remove_duplicate_candidates(candidates)
    }

    fn refine_intersection<F1,F2>(&self, curve1:&F1, curve2:&F2, initial_t1:f64, initial_t2:f64) -> Option<IntersectionCandidate>
    where F1: Fn(f64)->Point2D, F2: Fn(f64)->Point2D {
        let h = 1e-8;
        let system = |t1:f64, t2:f64| {
            let p1 = curve1(t1); let p2 = curve2(t2);
            let f1 = p1.x() - p2.x();
            let f2 = p1.y() - p2.y();
            let p1_dt = curve1(t1 + h); let p2_dt = curve2(t2 + h);
            let df1_dt1 = (p1_dt.x() - p1.x())/h; let df1_dt2 = -(p2_dt.x() - p2.x())/h;
            let df2_dt1 = (p1_dt.y() - p1.y())/h; let df2_dt2 = -(p2_dt.y() - p2.y())/h;
            (f1, f2, [[df1_dt1, df1_dt2],[df2_dt1, df2_dt2]])
        };
        if let Ok(((t1,t2), conv)) = self.newton_solver.solve_2d(system, (initial_t1, initial_t2)) { if conv.converged { let inter = curve1(t1); let ver = curve2(t2); let distance = inter.distance_to(&ver).value(); return Some(IntersectionCandidate { point: inter, parameter: t1, distance, confidence: 1.0/(1.0+conv.final_error)}); } }
        None
    }

    fn remove_duplicate_candidates(&self, mut candidates: Vec<IntersectionCandidate>) -> Vec<IntersectionCandidate> {
        candidates.sort_by(|a,b| a.parameter.partial_cmp(&b.parameter).unwrap());
        let mut unique = Vec::new();
        for c in candidates { let dup = unique.iter().any(|u:&IntersectionCandidate| u.point.distance_to(&c.point).value() < self.tolerance.linear ); if !dup { unique.push(c); } }
        unique
    }
}
