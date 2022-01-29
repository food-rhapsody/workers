import { parse, stringify } from '@iarna/toml';
import { readFile, writeFile } from 'fs/promises';
import path from 'path';

export async function readWranglerConfig(workingDir: string, configFileName = 'wrangler.toml') {
  const data = await readFile(path.join(workingDir, configFileName), 'utf8');

  return parse.async(data);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function writeWranglerConfig(workingDir: string, configFileName: string, data: any) {
  await writeFile(path.join(workingDir, configFileName), stringify(data), 'utf8');
}
