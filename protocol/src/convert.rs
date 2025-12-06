use nalgebra::{UnitVector2, Vector2, Vector3, Point3};

impl From<crate::Vector2> for Vector2<f32> {
    fn from(value: crate::Vector2) -> Self {
        Vector2::new(value.x, value.y)
    }
}

impl From<Vector2<f32>> for crate::Vector2 {
    fn from(value: Vector2<f32>) -> Self {
        crate::Vector2 { x: value.x, y: value.y }
    }
}

impl TryFrom<crate::Vector2> for UnitVector2<f32> {
    type Error = ();

    fn try_from(value: crate::Vector2) -> Result<Self, Self::Error> {
        if value.x == 0.0 && value.y == 0.0 {
            return Err(());
        }

        Ok(UnitVector2::new_normalize(Vector2::new(value.x, value.y)))
    }
}

impl From<UnitVector2<f32>> for crate::Vector2 {
    fn from(value: UnitVector2<f32>) -> Self {
        crate::Vector2 { x: value.x, y: value.y }
    }
}

impl From<crate::Vector3> for Point3<f32> {
    fn from(value: crate::Vector3) -> Self {
        Point3::new(value.x, value.y, value.z)
    }
}

impl From<Point3<f32>> for crate::Vector3 {
    fn from(value: Point3<f32>) -> Self {
        crate::Vector3 { x: value.x, y: value.y, z: value.z }
    }
}

impl From<crate::Vector3> for Vector3<f32> {
    fn from(value: crate::Vector3) -> Self {
        Vector3::new(value.x, value.y, value.z)
    }
}

impl From<Vector3<f32>> for crate::Vector3 {
    fn from(value: Vector3<f32>) -> Self {
        crate::Vector3 { x: value.x, y: value.y, z: value.z }
    }
}
