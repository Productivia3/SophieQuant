import type { VerdictEstado } from '../lib/api'

/**
 * Presentacion de los estados de veredicto. Vive fuera del componente para que
 * el modulo del componente solo exporte componentes (Fast Refresh de Vite).
 */

export const estadoTone = (e: VerdictEstado): 'pass' | 'fail' | 'running' | 'pending' =>
  e === 'pass' ? 'pass' : e === 'fail' ? 'fail' : e === 'warn' ? 'running' : 'pending'

export const estadoTexto = (e: VerdictEstado) =>
  e === 'pass' ? 'PASSED' : e === 'fail' ? 'FAILED' : e === 'warn' ? 'AVISO' : 'PENDING'

/** Orden de gravedad: de peor a mejor. */
export const ordenEstados: VerdictEstado[] = ['fail', 'pending', 'warn', 'pass']
