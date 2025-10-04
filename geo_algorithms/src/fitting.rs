use geo_core::{Point2D, Vector2D, ToleranceContext, Vector};

pub struct LeastSquaresFitter { tolerance: ToleranceContext }
impl LeastSquaresFitter { pub fn new(tolerance: ToleranceContext)->Self { Self { tolerance } } }

impl LeastSquaresFitter {
    pub fn fit_circle(&self, points:&[Point2D]) -> Result<(Point2D,f64), String> {
        if points.len() < 3 { return Err("Need at least 3 points for circle fitting".into()); }
        let mut s_x=0.0; let mut s_y=0.0; let mut s_x2=0.0; let mut s_y2=0.0; let mut s_xy=0.0; let mut s_x3=0.0; let mut s_y3=0.0; let mut s_x1y2=0.0; let mut s_x2y1=0.0;
        for p in points { let x=p.x().value(); let y=p.y().value(); let x2=x*x; let y2=y*y; s_x+=x; s_y+=y; s_x2+=x2; s_y2+=y2; s_xy+=x*y; s_x3+=x2*x; s_y3+=y2*y; s_x1y2+=x*y2; s_x2y1+=x2*y; }
        let n = points.len() as f64;
        let m11=s_x2; let m12=s_xy; let m13=s_x; let m21=s_xy; let m22=s_y2; let m23=s_y; let m31=s_x; let m32=s_y; let m33=n;
        let b1=-(s_x3 + s_x1y2); let b2=-(s_x2y1 + s_y3); let b3=-(s_x2 + s_y2);
        let det = m11*(m22*m33 - m23*m32) - m12*(m21*m33 - m23*m31) + m13*(m21*m32 - m22*m31);
        if det.abs() < self.tolerance.parametric { return Err("Degenerate point set".into()); }
        let det_d = b1*(m22*m33 - m23*m32) - m12*(b2*m33 - m23*b3) + m13*(b2*m32 - m22*b3);
        let det_e = m11*(b2*m33 - m23*b3) - b1*(m21*m33 - m23*m31) + m13*(m21*b3 - b2*m31);
        let det_f = m11*(m22*b3 - b2*m32) - m12*(m21*b3 - b2*m31) + b1*(m21*m32 - m22*m31);
        let d=det_d/det; let e=det_e/det; let f=det_f/det;
        let cx = -0.5*d; let cy = -0.5*e; let r_sq = cx*cx + cy*cy - f; if r_sq < -self.tolerance.parametric { return Err("Negative radius squared".into()); }
        let radius = if r_sq <= 0.0 { 0.0 } else { r_sq.sqrt() }; let center = Point2D::from_f64(cx, cy);
        Ok((center, radius))
    }

    pub fn fit_line(&self, points:&[Point2D]) -> Result<(Point2D, Vector2D), String> {
        if points.len() < 2 { return Err("Need at least 2 points for line fitting".into()); }
        let n=points.len() as f64; let mut sum_x=0.0; let mut sum_y=0.0; let mut sum_xy=0.0; let mut sum_x2=0.0;
        for p in points { let x=p.x().value(); let y=p.y().value(); sum_x+=x; sum_y+=y; sum_xy+=x*y; sum_x2+=x*x; }
        let denom = n*sum_x2 - sum_x*sum_x; if denom.abs() < self.tolerance.parametric { let avg_x = sum_x/n; let avg_y = sum_y/n; return Ok((Point2D::from_f64(avg_x, avg_y), Vector2D::from_f64(0.0,1.0))); }
        let slope = (n*sum_xy - sum_x*sum_y)/denom; let intercept = (sum_y - slope*sum_x)/n; let point = Point2D::from_f64(0.0, intercept); let dir = Vector2D::from_f64(1.0, slope); let tol = ToleranceContext::standard(); let dir = dir.normalize(&tol).unwrap_or(Vector2D::from_f64(1.0,0.0)); Ok((point, dir))
    }
}
