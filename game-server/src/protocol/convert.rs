use nalgebra::{UnitVector2, Vector2, Vector3, Point3};

impl From<crate::protocol::Vector2> for Vector2<f32> {
    fn from(value: crate::protocol::Vector2) -> Self {
        Vector2::new(value.x, value.y)
    }
}

impl From<Vector2<f32>> for crate::protocol::Vector2 {
    fn from(value: Vector2<f32>) -> Self {
        crate::protocol::Vector2 { x: value.x, y: value.y }
    }
}

impl TryFrom<crate::protocol::Vector2> for UnitVector2<f32> {
    type Error = ();

    fn try_from(value: crate::protocol::Vector2) -> Result<Self, Self::Error> {
        if value.x == 0.0 && value.y == 0.0 {
            return Err(());
        }

        Ok(UnitVector2::new_normalize(Vector2::new(value.x, value.y)))
    }
}

impl From<UnitVector2<f32>> for crate::protocol::Vector2 {
    fn from(value: UnitVector2<f32>) -> Self {
        crate::protocol::Vector2 { x: value.x, y: value.y }
    }
}

impl From<crate::protocol::Vector3> for Point3<f32> {
    fn from(value: crate::protocol::Vector3) -> Self {
        Point3::new(value.x, value.y, value.z)
    }
}

impl From<Point3<f32>> for crate::protocol::Vector3 {
    fn from(value: Point3<f32>) -> Self {
        crate::protocol::Vector3 { x: value.x, y: value.y, z: value.z }
    }
}

impl From<crate::protocol::Vector3> for Vector3<f32> {
    fn from(value: crate::protocol::Vector3) -> Self {
        Vector3::new(value.x, value.y, value.z)
    }
}

impl From<Vector3<f32>> for crate::protocol::Vector3 {
    fn from(value: Vector3<f32>) -> Self {
        crate::protocol::Vector3 { x: value.x, y: value.y, z: value.z }
    }
}

impl From<crate::protocol::Uuid> for uuid::Uuid {
    fn from(value: crate::protocol::Uuid) -> Self {
        uuid::Uuid::from_u64_pair(value.high, value.low)
    }
}

impl From<uuid::Uuid> for crate::protocol::Uuid {
    fn from(value: uuid::Uuid) -> Self {
        let (high, low) = value.as_u64_pair();
        crate::protocol::Uuid { high, low }
    }
}
