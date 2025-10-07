// camera.rs
use raylib::prelude::*;

pub struct Camera {
    pub eye: Vector3,     // donde esta la camara en el mundo, ejemplo (7,100,10)
    pub center: Vector3,  // que mira la camara (mario), ejemplo (7,100,5)
    pub up: Vector3,      // donde esta arriba
    pub forward: Vector3,
    pub right: Vector3,
    changed: bool,        // para optimizar actualizaciones
}

impl Camera {
    pub fn new(eye: Vector3, center: Vector3, up: Vector3) -> Self {
        let mut camera = Camera {
            eye,
            center,
            up,
            forward: Vector3::zero(),
            right: Vector3::zero(),
            changed: true,
        };

        camera.update_basis();
        camera
    }

    pub fn update_basis(&mut self) {
        self.forward = (self.center - self.eye).normalized();
        self.right = self.forward.cross(self.up).normalized();
        self.up = self.right.cross(self.forward);
        self.changed = true;
    }

    /// Rota la cámara alrededor del punto center (movimiento orbital)
    pub fn orbit(&mut self, yaw: f32, pitch: f32) {
        let relative_pos = self.eye - self.center;

        let radius = relative_pos.length();

        let current_yaw = relative_pos.z.atan2(relative_pos.x);
        let current_pitch = (relative_pos.y / radius).asin();

        // coordenadas esféricas
        let new_yaw = current_yaw + yaw;
        let new_pitch = (current_pitch + pitch).clamp(-1.5, 1.5);

        let pitch_cos = new_pitch.cos();
        let pitch_sin = new_pitch.sin();

        // x = r * cos(a) * cos(b)
        // y = r * sin(a)
        // z = r * cos(a) * sin (b)
        let new_relative_pos = Vector3::new(
            radius * pitch_cos * new_yaw.cos(),
            radius * pitch_sin,
            radius * pitch_cos * new_yaw.sin(),
        );

        self.eye = self.center + new_relative_pos;

        self.update_basis();
    }

    /// Acerca o aleja la cámara hacia/desde el centro
    pub fn zoom(&mut self, amount: f32) {
        let forward = (self.center - self.eye).normalized();
        self.eye += forward * amount;
        self.update_basis();
    }

    /// Verifica si la cámara ha cambiado desde la última vez que se llamó esta función
    pub fn is_changed(&mut self) -> bool {
        let changed = self.changed;
        self.changed = false;
        changed
    }

    /// Transforma un vector del espacio de cámara al espacio mundial usando los vectores base
    pub fn basis_change(&self, p: &Vector3) -> Vector3 {
        // Conversión de espacio de cámara a espacio mundial
        // En espacio de cámara:
        // - X apunta a la derecha
        // - Y apunta arriba
        // - Z apunta hacia atrás (opuesto a donde mira la cámara)
        Vector3::new(
            p.x * self.right.x + p.y * self.up.x - p.z * self.forward.x,
            p.x * self.right.y + p.y * self.up.y - p.z * self.forward.y,
            p.x * self.right.z + p.y * self.up.z - p.z * self.forward.z,
        )
    }
}