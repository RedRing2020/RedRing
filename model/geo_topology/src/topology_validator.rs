use crate::tolerance::ResolvedTopologyToleranceBudget;
use crate::{Edge, TopologyToleranceSettings, Wire};
use geo_contracts::Scalar;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeValidationReport<T: Scalar> {
    pub binding_consistent: bool,
    pub ideal_endpoint_consistent: bool,
    pub evaluated_endpoint_consistent: bool,
    pub edge_tolerances: TopologyToleranceSettings<T>,
}

impl<T: Scalar> EdgeValidationReport<T> {
    pub fn is_valid(&self) -> bool {
        self.binding_consistent
            && self.ideal_endpoint_consistent
            && self.evaluated_endpoint_consistent
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WireValidationReport<T: Scalar> {
    pub edge_reports: Vec<EdgeValidationReport<T>>,
    pub shared_vertex_consistent: Vec<bool>,
    pub shared_tolerance: T,
}

impl<T: Scalar> WireValidationReport<T> {
    pub fn is_valid(&self) -> bool {
        self.edge_reports.iter().all(EdgeValidationReport::is_valid)
            && self.shared_vertex_consistent.iter().all(|status| *status)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologyValidator<T: Scalar> {
    tolerances: TopologyToleranceSettings<T>,
}

impl<T: Scalar> TopologyValidator<T> {
    pub fn new(tolerances: TopologyToleranceSettings<T>) -> Self {
        Self { tolerances }
    }

    pub fn tolerances(&self) -> TopologyToleranceSettings<T> {
        self.tolerances
    }

    pub fn validate_edge(&self, edge: &Edge<T>) -> EdgeValidationReport<T> {
        let budget = self.tolerances.resolve_validator_budget();

        EdgeValidationReport {
            binding_consistent: edge.is_binding_consistent(budget.edge.bind_tolerance),
            ideal_endpoint_consistent: edge
                .is_ideal_endpoint_consistent(budget.edge.ideal_tolerance),
            evaluated_endpoint_consistent: edge
                .is_evaluated_endpoint_consistent(budget.edge.eval_tolerance),
            edge_tolerances: self.tolerances,
        }
    }

    pub fn validate_wire(&self, wire: &Wire<T>) -> WireValidationReport<T> {
        let budget = self.tolerances.resolve_validator_budget();
        let edge_reports = wire
            .edges()
            .iter()
            .map(|edge| self.validate_edge(edge))
            .collect::<Vec<_>>();
        let shared_vertex_consistent = shared_vertex_consistency(wire, budget);

        WireValidationReport {
            edge_reports,
            shared_vertex_consistent,
            shared_tolerance: budget.shared_tolerance,
        }
    }
}

fn shared_vertex_consistency<T: Scalar>(
    wire: &Wire<T>,
    budget: ResolvedTopologyToleranceBudget<T>,
) -> Vec<bool> {
    if wire.edges().len() < 2 {
        return Vec::new();
    }

    (0..wire.edges().len() - 1)
        .map(|index| {
            let end = wire.edges()[index].oriented_constraint_end_point();
            let next_start = wire.edges()[index + 1].oriented_constraint_start_point();
            end.distance_to(&next_start) <= budget.shared_tolerance
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{TopologyValidator, WireValidationReport};
    use crate::{CurveRef, Point3D, TopoInfiniteLine3D, TopoLineSegment3D, Vertex, Wire};
    use std::sync::Arc;

    fn make_edge(start: Point3D<f64>, end: Point3D<f64>) -> crate::Edge<f64> {
        let v0 = Arc::new(Vertex::new(start));
        let v1 = Arc::new(Vertex::new(end));
        let line = TopoLineSegment3D::new(start, end).unwrap();

        crate::Edge::new(
            v0,
            v1,
            CurveRef::Line(line),
            (line.start_param(), line.end_param()),
        )
        .unwrap()
    }

    fn make_offset_support_edge(start: Point3D<f64>, end: Point3D<f64>) -> crate::Edge<f64> {
        let support_start = Point3D::new(start.x(), start.y(), 0.0);
        let support_end = Point3D::new(end.x(), end.y(), 0.0);
        let support_line = TopoInfiniteLine3D::from_two_points(support_start, support_end).unwrap();
        let line =
            TopoLineSegment3D::from_support_line_and_constraint_points(support_line, start, end)
                .unwrap();

        crate::Edge::new(
            Arc::new(Vertex::new(start)),
            Arc::new(Vertex::new(end)),
            CurveRef::Line(line),
            (line.start_param(), line.end_param()),
        )
        .unwrap()
    }

    #[test]
    fn validator_reports_edge_consistency_breakdown() {
        let start = Point3D::new(0.0, 0.0, 0.1);
        let end = Point3D::new(2.0, 0.0, 0.1);
        let edge = make_offset_support_edge(start, end);
        let validator =
            TopologyValidator::new(crate::TopologyToleranceSettings::new(1.0e-9, 1.0e-9));

        let report = validator.validate_edge(&edge);

        assert!(report.binding_consistent);
        assert!(!report.ideal_endpoint_consistent);
        assert!(!report.evaluated_endpoint_consistent);
        assert!(!report.is_valid());
    }

    #[test]
    fn validator_reports_wire_shared_vertex_failures() {
        let e1 = make_edge(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0));
        let e2 = make_edge(Point3D::new(1.1, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0));
        let wire = Wire::new_with_tolerances(vec![e1, e2], 1.0e-9, 0.2).unwrap();
        let validator = TopologyValidator::new(crate::TopologyToleranceSettings::new(1.0e-9, 0.05));

        let report: WireValidationReport<f64> = validator.validate_wire(&wire);

        assert_eq!(report.shared_vertex_consistent, vec![false]);
        assert!(!report.is_valid());
    }
}
