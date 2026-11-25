//! Hittable trait and HitRecord

use super::math::{Ray, Vector3};
/// HitRecord.
///
/// # Examples
///
/// ```
/// use dotmax::raytracer::{HitRecord, Ray, Vector3};
/// let ray = Ray::new(Vector3::zero(), Vector3::new(0.0, 0.0, -1.0));
/// let hit = HitRecord::new(Vector3::zero(), Vector3::new(0.0, 0.0, 1.0), 1.5, &ray);
/// assert!(hit.front_face);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitRecord {
    pub point: Vector3,
    pub normal: Vector3,
    pub t: f32,
    pub front_face: bool,
}

impl HitRecord {
    /// new.
    ///
    /// # Examples
    ///
    /// ```
    /// use dotmax::raytracer::{HitRecord, Ray, Vector3};
    /// let ray = Ray::new(Vector3::zero(), Vector3::new(0.0, 0.0, -1.0));
    /// let hit = HitRecord::new(Vector3::zero(), Vector3::new(0.0, 0.0, 1.0), 1.5, &ray);
    /// assert!(hit.front_face);
    /// ```
    #[inline]
    pub fn new(point: Vector3, outward_normal: Vector3, t: f32, ray: &Ray) -> Self {
        let front_face = ray.direction.dot(&outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        Self {
            point,
            normal,
            t,
            front_face,
        }
    }
}
/// Hittable.
///
/// # Examples
///
/// ```
/// use dotmax::raytracer::{Hittable, Ray, Sphere, Vector3};
/// let sphere = Sphere::new(Vector3::new(0.0, 0.0, -3.0), 1.0);
/// let ray = Ray::new(Vector3::zero(), Vector3::new(0.0, 0.0, -1.0));
/// assert!(sphere.hit(&ray, 0.001, 100.0).is_some());
/// ```
pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}
