# Lienzo — portapapeles con línea de tiempo

Reemplazo del portapapeles de Windows: todo lo que copias (texto e imágenes) cae en un lienzo organizado por hora, día, mes y año.

## Usarlo

1. Descarga `Lienzo_x.y.z_x64-setup.exe` desde [Releases](https://github.com/dmore107/lienzo/releases/latest) e instálalo (no pide permisos de administrador; se instala en tu perfil).
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
| App de origen | Cada tarjeta muestra de qué programa se copió; también puedes buscar por él («chrome», «excel»…) |
| Sin duplicados | Si copias algo que ya estaba, sube al presente en lugar de repetirse |
| Iniciar con Windows | Bandeja → **Iniciar con Windows** (arranca oculto en la bandeja) |
| Actualizar | Cuando hay versión nueva aparece **Actualizar a x.y.z** arriba; también Bandeja → **Buscar actualizaciones** |

## Reuniones y capacitaciones

| Acción | Cómo |
|---|---|
| Grabar | Ícono del micrófono arriba: eliges Reunión o Capacitación y un título opcional. Siempre graba tu micrófono y el audio del PC |
| Grabar al instante | **Alt+Shift+R** o Bandeja → **Grabar reunión** (micrófono + audio del PC). El mismo atajo detiene |
| Pausar / detener | Barra roja sobre el tablero, con el tiempo y el nivel de cada fuente |
| Transcribir | Automático al terminar de grabar (o **Transcribir** en la tarjeta). Todo se procesa en tu equipo |
| Leer y escuchar | Doble clic en la tarjeta: transcripción con el audio; clic en una hora para escuchar ese momento |
| Copiar la transcripción | En el visor, botón de copiar |
| Modelo e idioma | Clic derecho en el tablero → **Ajustes de transcripción** (Rápido 57 MB · Equilibrado 181 MB · Preciso 547 MB) |
| Ver el audio | Clic derecho en la tarjeta → **Abrir la carpeta del audio** |

Se graban por separado tu micrófono (canal izquierdo) y lo que suena en el PC (canal derecho: Teams, Zoom, Meet, videos), en un WAV de 16 kHz (unos 230 MB por hora). Si Lienzo se cierra a mitad de una grabación, el audio se conserva hasta los últimos segundos. La transcripción marca qué dijiste **tú** y qué dijeron los **participantes** según el canal, y queda también en `transcripcion.txt` junto al audio. El modelo se descarga una sola vez desde Hugging Face y se verifica su huella SHA-256. El acta con resumen, decisiones y tareas llega en la próxima versión.

Avisa a los participantes que estás grabando. Si el micrófono no se oye, revisa en Configuración → Privacidad y seguridad → Micrófono que **las aplicaciones de escritorio** tengan acceso.

## Dónde quedan los datos

Solo en el equipo donde lo ejecutas:
`%LOCALAPPDATA%\com.dani.lienzo\` (base `lienzo.db`, carpeta `img` y carpeta `sesiones` con el audio de cada grabación).
Si instalas Lienzo en otro equipo, arranca vacío. Tu contenido nunca sale del equipo; lo único que Lienzo consulta en internet es si hay una versión nueva en GitHub.

El contenido que los gestores de contraseñas marcan como privado (KeePass, 1Password, Bitwarden…) no se guarda.

## Usar Win+V

Windows reserva Win+V para su propio historial. Para que Lienzo lo use:
Configuración → Sistema → Portapapeles → desactiva **Historial del portapapeles** y reinicia Lienzo (Salir desde la bandeja y abrir de nuevo). Alt+V funciona siempre.

## Desarrollo

Requiere Rust, las herramientas de compilación de Visual Studio, CMake y Node.js. Para que whisper.cpp funcione en cualquier CPU con AVX2, define antes `GGML_NATIVE=OFF` (lo hace GitHub Actions):

```
npm install
npm run dev      # abre Lienzo en modo desarrollo
npm run build    # genera el instalador en src-tauri/target/release/bundle/nsis
```

- `ui/index.html`, `ui/app.css`, `ui/app.js` — interfaz (HTML/CSS/JS, sin frameworks)
- `src-tauri/src/main.rs` — captura del portapapeles, comandos, bandeja y arranque
- `src-tauri/src/db.rs` — SQLite y migraciones numeradas (`PRAGMA user_version`)
- `src-tauri/src/win.rs` — API de Windows: portapapeles, foco, pegado, app de origen
- `src-tauri/src/rec.rs` — grabación de reuniones y capacitaciones (micrófono + audio del PC)
- `src-tauri/src/transcribe.rs` — transcripción local con whisper.cpp, modelos y cola de trabajo
- `src-tauri/src/updater.rs` — actualizaciones firmadas desde GitHub Releases

## Publicar una versión

1. Sube el número de versión en `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` y `package.json` (por ejemplo `0.3.0`).
2. Haz commit y crea la etiqueta: `git tag v0.3.0 && git push origin main v0.3.0`.
3. GitHub Actions compila el instalador, lo firma y lo publica en Releases con `latest.json`. Las copias instaladas lo detectan en las siguientes horas (o con Buscar actualizaciones).

Las actualizaciones se firman con la llave privada de `%USERPROFILE%\.tauri\lienzo.key`, guardada como secreto del repositorio (`TAURI_SIGNING_PRIVATE_KEY` y `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`). **Respáldala:** sin ella no se pueden publicar actualizaciones que las copias instaladas acepten.
