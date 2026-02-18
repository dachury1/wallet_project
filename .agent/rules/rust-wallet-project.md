---
trigger: always_on
---

# Reglas del Proyecto: Sistema de Compensación de Transacciones

Este documento define las reglas de arquitectura, diseño y desarrollo para el sistema de pagos multi-usuario.

## 1. Objetivo del Proyecto
Desarrollar un sistema de pagos robusto que maneje saldos y cuotas en tiempo real, garantizando la **consistencia** y el **rendimiento** bajo alta concurrencia.

## 2. Arquitectura del Sistema

### Estructura de Microservicios
El sistema se compone de dos servicios principales:
1.  **Servicio de Wallet**: Fuente de la verdad para balances y gestión de usuarios.
2.  **Servicio de Transacciones**: Registro de movimientos y orquestación de validaciones.

### Clean Architecture (Estructura Interna)
Cada microservicio debe seguir estricamente la separación de capas:

-   **`domain/`**:
    -   Contiene entidades (ej. `Balance`, `Usuario`) y reglas de negocio puras.
    -   *No debe tener dependencias externas* (sin `sqlx`, `axum`, etc.).
    -   Definición de errores de dominio con `thiserror`.

-   **`use_cases/`**:
    -   Orquesta la lógica de la aplicación (ej. `TransferirFondos`).
    -   Utiliza abstracciones (Traits) para acceder a datos y servicios externos.
    -   Inyección de dependencias mediante `Arc<dyn Repository>`.

-   **`infrastructure/`**:
    -   Implementación concreta de adaptadores.
    -   Base de Datos: Repositorios usando `SQLx` (PostgreSQL).
    -   API HTTP: Handlers de `Axum`.
    -   gRPC: Clientes y servidores `Tonic`.
    -   Manejo de errores con `anyhow`.

## 3. Stack Tecnológico

-   **Lenguaje**: Rust (Edición 2021 o superior).
-   **Web Framework**: `Axum` (sobre Tokio).
-   **Base de Datos**: `SQLx` (consultas verificadas en tiempo de compilación).
    -   *Opcional*: `SeaQuery` para consultas dinámicas estéticas si es necesario.
-   **Comunicación Inter-servicios**: gRPC con `Tonic`.
-   **Serialización**: `Serde`.

## 4. Estándares de Código y Diseño

### Inyección de Dependencias
-   Definir **Traits** para todos los repositorios en la capa de dominio o casos de uso.
-   Inyectar implementaciones concretas envueltas en `Arc<T>` para permitir concurrencia segura y facilitar el testing.

### Manejo de Errores
-   **Domain**: Usar `thiserror` para errores tipados y específicos del negocio.
-   **Infrastructure/App**: Usar `anyhow` para propagación de errores genéricos y contexto.

### Patrones de Diseño
-   **Builder Pattern**: Utilizarlo obligatoriamente para construir entidades de dominio complejas, asegurando estados válidos desde la creación.
-   **NewType Pattern**: Usar tipos envoltorios (ej. `struct UserId(Uuid)`) para evitar confusión de tipos primitivos.

## 5. Estrategia de Testing

Se debe mantener una alta cobertura siguiendo la pirámide de testing:

### Unit Tests
-   **Domain**: Testear exhaustivamente reglas de negocio y validaciones.
-   **Use Cases**: Testear la lógica de aplicación usando **Mocks** de los repositorios y servicios (simulando la infraestructura).

### Integration Tests
-   **Infrastructure**: Testear repositorios reales contra una base de datos de prueba (usando contenedores Docker si es posible) para validar consultas SQL.

### E2E Tests
-   Probar flujos completos levantando ambos microservicios y simulando comunicación gRPC real.

## 6. Observabilidad y Documentación

-   **Tracing**: Implementar logs estructurados (`tracing` + `tracing-subscriber`) en todas las capas para rastrear solicitudes entre microservicios.
-   **Documentación**:
    -   Todas las funciones y estructuras públicas deben tener documentación (`///`).
    -   Generar documentación con `cargo doc`.

### Benchmarking (Opcional)
-   Implementar benchmarks con `criterion` o herramientas nativas de Rust para medir el rendimiento de secciones críticas (validaciones de saldos, etc.).

