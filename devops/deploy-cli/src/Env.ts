export type Env = 'live' | 'test';
export const envs: Env[] = ['live', 'test'];

export function parseEnv(val: unknown): Env | undefined {
  if (envs.some(x => x === val)) {
    return val as Env;
  }
  return undefined;
}
