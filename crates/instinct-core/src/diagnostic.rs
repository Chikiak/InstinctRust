//! Diagnóstico con archivo, línea y mensaje.
//! Formato de texto: `archivo:línea: mensaje`.

/// Un diagnóstico producido por compilación, carga o ejecución.
/// La línea siempre se cuenta desde 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub archivo: String,
    pub linea: usize,
    pub mensaje: String,
}

impl Diagnostic {
    /// Construye un nuevo diagnóstico.
    pub fn new(archivo: impl Into<String>, linea: usize, mensaje: impl Into<String>) -> Self {
        Diagnostic {
            archivo: archivo.into(),
            linea,
            mensaje: mensaje.into(),
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.archivo, self.linea, self.mensaje)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_display_format() {
        let d = Diagnostic::new("criaturas/walker.ins", 42, "etiqueta 'fin' no definida");
        assert_eq!(
            d.to_string(),
            "criaturas/walker.ins:42: etiqueta 'fin' no definida"
        );
        assert_eq!(d.archivo, "criaturas/walker.ins");
        assert_eq!(d.linea, 42);
        assert_eq!(d.mensaje, "etiqueta 'fin' no definida");
    }
}
