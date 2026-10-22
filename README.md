# Lienzo — portapapeles con línea de tiempo

Reemplazo del portapapeles de Windows: todo lo que copias (texto e imágenes) cae en un lienzo organizado por hora, día, mes y año.

## Usarlo

1. Abre `Lienzo.exe` (no se instala, no toca el registro).
2. Queda en la bandeja del sistema (junto al reloj).
3. Ábrelo con **Alt+V** (o **Win+V** si desactivas el historial del portapapeles de Windows).
4. Copia cualquier cosa con Ctrl+C: aparece en el lienzo automáticamente.

| Acción | Cómo |
|---|---|
| Pegar en la app donde estabas | Botón ⤓ en la tarjeta, clic en un fijado, o Enter en la búsqueda |
| Buscar | Ctrl+K o empieza a escribir |
| Editar / corregir redacción | Doble clic en la tarjeta |
| Fijar | Arrastra la tarjeta al panel Fijados |
| Reordenar fijados | Arrastra un fijado arriba o abajo dentro del panel |
| Menú de opciones | Clic derecho sobre una nota, un fijado o el tablero |
| Ordenar el tablero | Clic derecho → Hora ↑/↓, A → Z, Z → A o Modo libre (también con el botón junto al conteo) |
| Modo libre | Arrastra cualquier nota a donde quieras; queda guardada en ese lugar |
| Tono de una nota | Clic derecho → Tono (gris, grafito, azul, verde, ámbar, rosa, lila) |
| Eliminar | Papelera (se vacía sola a los 30 días) |
| Zoom de la línea de tiempo | Rueda del mouse sobre ella |
| Ocultar | Esc o clic fuera (usa la chincheta de arriba para mantenerla abierta) |
| Pausar captura | Botón ‖ arriba o desde la bandeja |
| Opacidad | Botón ◐ arriba (30 % a 100 %) |
| Vista compacta | Botón de encoger arriba, o haz la ventana angosta: muestra la hora actual con lo que vas copiando |
| Volver a la vista completa | Botón de agrandar en la vista compacta |
| Colores de Windows | Tercer círculo de temas: usa el color de acento configurado en Windows |

## Dónde quedan los datos

Solo en el equipo donde lo ejecutas:
`%LOCALAPPDATA%\com.dani.lienzo\` (base `lienzo.db` + carpeta `img`).
Si copias `Lienzo.exe` a otro equipo, arranca vacío. Nada se envía a internet.

El contenido que los gestores de contraseñas marcan como privado (KeePass, 1Password, Bitwarden…) no se guarda.

## Usar Win+V

Windows reserva Win+V para su propio historial. Para que Lienzo lo use:
Configuración → Sistema → Portapapeles → desactiva **Historial del portapapeles** y reinicia Lienzo (Salir desde la bandeja y abrir de nuevo). Alt+V funciona siempre.

## Que arranque con Windows (opcional)

Win+R → `shell:startup` → pega ahí un acceso directo a `Lienzo.exe`.

## Compilar desde el código

Requiere Rust y las herramientas de compilación de Visual Studio:

```
cd src-tauri
cargo build --release
```

El ejecutable queda en `src-tauri/target/release/lienzo.exe`.

- `ui/index.html` — interfaz (HTML/CSS/JS, sin frameworks)
- `src-tauri/src/main.rs` — captura del portapapeles, base de datos SQLite, atajos, bandeja
