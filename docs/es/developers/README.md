# Documentación para desarrolladores

**Español** · [English](../../en/developers/README.md)

Esta sección es para quien quiere compilar marks, entender cómo está hecho o contribuir. La documentación de uso está en el [índice general](../README.md).

marks v1.0 se publica solo para macOS. Apple Silicon es la única plataforma probada; Intel se compila sin probar. Windows y Linux llegarán más adelante.

## Páginas

| Página | Contenido |
|---|---|
| [Arquitectura](architecture.md) | Las piezas del repositorio y el recorrido de un bookmark |
| [Compilar y verificar](building.md) | Requisitos, desarrollo, build de release y `scripts/check.sh` |
| [Reglas del repositorio](rules.md) | Lo que todo cambio debe respetar |
| [Idiomas y textos](i18n.md) | Cómo añadir un texto o un idioma |
| [Migraciones](migrations.md) | Cómo cambia el esquema de la base de datos |
| [API local](local-api.md) | La API HTTP que usa la extensión del navegador |
| [Contribuir](contributing.md) | Issues, pull requests, commits y verificación |
| [Publicar una versión](release.md) | Esqueleto del proceso de release |

## Otros documentos del repositorio

- [`AGENTS.md`](../../../AGENTS.md): mapa del repositorio y reglas, en formato breve. Está en español.
- [`DESIGN.md`](../../../DESIGN.md): el diseño visual y sus tokens. Hay que leerlo antes de tocar la interfaz.
