# Documento de Diseño: Servicio de Auditoría y Reconciliación

**Estado:** Planeado (No Implementado Actualmente)
**Propósito:** Garantizar la consistencia asíncrona entre las transacciones completadas y los saldos reales de las wallets.

## 1. El Problema a Resolver
En una arquitectura de microservicios, el `transaction_service` puede registrar una transacción como `COMPLETED` si asumió que la comunicación con el `wallet_service` fue exitosa. Sin embargo, errores de red, un crash de base de datos o *rollbacks* silenciosos en el `wallet_service` pueden dejarnos en un estado inconsistente:
- **Ejemplo**: El `transaction_service` marca que el `Wallet A` envió $10 al `Wallet B`. Pero el `wallet_service` debido a un fallo nunca procesó la actualización.
- Esto significa que hay fuga o creación de dinero no contabilizado.

## 2. Concepto de Reconciliación Contable
Para solucionar esto, se propone implementar un **Servicio de Auditoría**.  
El objetivo de este proceso es tomar las transacciones marcadas como `COMPLETED` (la fuente de la verdad para movimientos históricos) y compararlas matemáticamente contra el saldo real ("Snapshot") reportado por la Billetera.

Por cada Wallet activa, el servicio debe confirmar la siguiente ecuación matemática fundamental:
```text
[Fondo Inicial de la Wallet]
+ [Suma de DEPOSITOS exitosos]
+ [Suma de TRANSFERENCIAS ENTRANTES]
- [Suma de RETIROS exitosos]
- [Suma de TRANSFERENCIAS SALIENTES]
=======================================
DEBE SER IGUAL A -> Saldo Actual en Wallet
```

## 3. Posibles Opciones de Arquitectura (Trade-offs)

### Opción 1: Microservicio Independiente (`audit_service`) - *Recomendado para Producción a Escala*
Crear un nuevo repositorio de Rust que se comporta como un observador.
- **Flujo**:
  1. Corre como un _CronJob_ (ej. todas las noches a las 2:00 AM).
  2. Lee la base de datos de Wallets (sólo lectura) para obtener la lista de usuarios.
  3. Ejecuta sentencias SQL `SUM()` agrupadas sobre la base de datos de Transacciones.
  4. Compara ambos resultados. Si hay una diferencia (descuadre), genera un registro en una tabla `audit_discrepancies` y enciende alarmas críticas (PagerDuty, Slack).
- **Ventaja**: Cero impacto de rendimiento sobre el flujo de cobros de los clientes. Separación dura de responsabilidades. Cero Single Point of Failure.

### Opción 2: Background Worker en el propio `transaction_service`
Reutilizando el flujo actual de tareas en background de Tokio.
- **Flujo**:
  1. `tokio::spawn` ejecuta un worker intermitente que levanta batches de Wallets.
  2. Suma las transacciones de las wallets a lo largo del tiempo.
  3. Hace llamadas por vía gRPC (`GetBalance`) al `wallet_service` para obtener el saldo vivo.
  4. Efectúa la comparación y arroja un log de `ERROR`.
- **Ventaja**: Más fácil y barato de implementar inicialmene, puesto que no requiere aprovisionar ni hostear una pieza de infraestructura diferente.
- **Desventaja**: Tráfico excesivo a través de gRPC si hay gran volumen de clientes, impactando el procesamiento diario de transacciones concurrentes en la API Pública.

## 4. Requisitos Previos para la Implementación Futura
Antes de crear este servicio, se requiere lo siguiente de los otros equipos:
1. **Wallet Service**: Debe exponer un endpoint (gRPC/HTTP) estilo `GetBalance(wallet_id)` capaz de responder extremadamente rápido, u ofrecer un Read-Replica SQL.
2. **Sistema de Monitoreo**: Un destino claro a dónde enviar los eventos `DiscrepancyFound` (DataDog, Kibana, Slack Webhook).
