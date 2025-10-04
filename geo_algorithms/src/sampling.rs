use geo_core::{Point2D, ToleranceContext};

#[derive(Debug, Clone)]
pub struct SamplingResult<T> { pub points: Vec<T>, pub parameter_values: Vec<f64>, pub quality_metrics: QualityMetrics }
#[derive(Debug, Clone)]
pub struct QualityMetrics { pub uniformity_score: f64, pub coverage_ratio: f64, pub density_variance: f64 }
#[derive(Debug, Clone)]
pub struct IntersectionCandidate { pub point: Point2D, pub parameter: f64, pub distance: f64, pub confidence: f64 }

pub struct AdaptiveSampler { tolerance: ToleranceContext, max_recursion: usize, min_samples: usize }
impl AdaptiveSampler { pub fn new(tolerance: ToleranceContext)->Self { Self { tolerance, max_recursion:8, min_samples:10 } }
    pub fn sample_curve_adaptive<F,G>(&self, evaluator:F, curvature_fn:G, start:f64, end:f64) -> SamplingResult<Point2D>
        where F: Fn(f64)->Point2D, G: Fn(f64)->f64 {
        let mut points=Vec::new(); let mut parameters=Vec::new(); self.sample_recursive(&evaluator,&curvature_fn,start,end,0,&mut points,&mut parameters);
        let quality = self.calculate_quality_metrics(&points,&parameters); SamplingResult { points, parameter_values: parameters, quality_metrics: quality }
    }
    fn sample_recursive<F,G>(&self, evaluator:&F, curvature_fn:&G, start:f64, end:f64, depth:usize, points:&mut Vec<Point2D>, parameters:&mut Vec<f64>) where F: Fn(f64)->Point2D, G: Fn(f64)->f64 {
        if depth >= self.max_recursion { let mid=(start+end)*0.5; points.push(evaluator(mid)); parameters.push(mid); return; }
        let mid=(start+end)*0.5; let curvature=curvature_fn(mid); let subdiv = curvature.abs()>self.tolerance.curvature || (end-start)>self.tolerance.parametric;
        if subdiv && points.len()<1000 { self.sample_recursive(evaluator,curvature_fn,start,mid,depth+1,points,parameters); self.sample_recursive(evaluator,curvature_fn,mid,end,depth+1,points,parameters); return; }
        points.push(evaluator(mid)); parameters.push(mid);
    }
    fn calculate_quality_metrics(&self, points:&[Point2D], parameters:&[f64]) -> QualityMetrics { if points.len()<2 { return QualityMetrics { uniformity_score:0.0, coverage_ratio:0.0, density_variance:0.0 }; } let mut distances=Vec::new(); for i in 1..points.len(){ let d=points[i].distance_to(&points[i-1]).value(); distances.push(d);} let mean: f64 = distances.iter().sum::<f64>()/distances.len() as f64; let var: f64 = distances.iter().map(|d|(d-mean).powi(2)).sum::<f64>()/distances.len() as f64; let uniformity = if var>0.0 { 1.0/(1.0 + var.sqrt()/mean) } else { 1.0 }; QualityMetrics { uniformity_score: uniformity, coverage_ratio: (parameters.len() as f64)/self.min_samples as f64, density_variance: var } }
}

pub struct PoissonDiskSampler { radius: f64, k: usize }
impl PoissonDiskSampler { pub fn new(radius:f64)->Self { Self { radius, k:30 } }
    pub fn sample_uniform_2d(&self, bounds:(Point2D,Point2D), initial_seed:Option<Point2D>) -> Vec<Point2D> { let mut rng=SimpleRng::new(42); let mut points=Vec::new(); let mut active=Vec::new(); let first = initial_seed.unwrap_or_else(|| { let cx=(bounds.0.x()+bounds.1.x())*0.5; let cy=(bounds.0.y()+bounds.1.y())*0.5; Point2D::from_f64(cx,cy)}); points.push(first); active.push(0); while !active.is_empty() { let idx=(rng.next_f64()*active.len() as f64) as usize; let pidx = active[idx]; let p = points[pidx]; let mut found=false; for _ in 0..self.k { let angle=rng.next_f64()*2.0*std::f64::consts::PI; let distance=self.radius*(1.0 + rng.next_f64()); let nx=p.x()+distance*angle.cos(); let ny=p.y()+distance*angle.sin(); let np=Point2D::from_f64(nx,ny); if self.is_valid_point(&np,&points,bounds) { points.push(np); active.push(points.len()-1); found=true; break; } } if !found { active.swap_remove(idx); } } points }
    fn is_valid_point(&self, p:&Point2D, existing:&[Point2D], bounds:(Point2D,Point2D)) -> bool { if p.x()<bounds.0.x() || p.x()>bounds.1.x() || p.y()<bounds.0.y() || p.y()>bounds.1.y() { return false; } for e in existing { if e.distance_to(p).value() < self.radius { return false; } } true }
}

struct SimpleRng { state: u64 }
impl SimpleRng { fn new(seed:u64)->Self { Self { state:seed } } fn next_u64(&mut self)->u64 { self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1); self.state } fn next_f64(&mut self)->f64 { (self.next_u64() >> 11) as f64 / (1u64<<53) as f64 } }
