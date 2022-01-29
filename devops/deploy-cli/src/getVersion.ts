import execa from 'execa';
import { readFile } from 'fs/promises';
import path from 'path';
import { Env } from './Env';

interface GetVersionParams {
  workingDir: string;
  env: Env;
}

export async function getVersion({ workingDir, env }: GetVersionParams) {
  const pkgRaw = await readFile(path.join(workingDir, 'package.json'), 'utf8');
  const { version } = JSON.parse(pkgRaw);

  if (env === 'test') {
    const commitId = await getCurrentCommitId();
    return `${version}-${commitId.slice(0, 7)}`;
  }

  return version;
}

async function getCurrentCommitId() {
  const { stdout } = await execa('git', ['rev-parse', 'HEAD'], {
    stripFinalNewline: true,
  });

  return stdout.trim();
}
