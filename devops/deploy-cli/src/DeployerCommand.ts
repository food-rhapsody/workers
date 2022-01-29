import { Command, Option } from 'clipanion';
import execa from 'execa';
import { isEnum } from 'typanion';
import { Env, envs, parseEnv } from './Env';
import { getVersion } from './getVersion';
import { readWranglerConfig, writeWranglerConfig } from './wranglerConfig';

export class DeployerCommand extends Command {
  readonly workingDir = Option.String('-w,--working-dir');
  readonly env = Option.String('-e,--env', {
    validator: isEnum(envs),
  });

  async execute() {
    const workingDir = this.workingDir ?? process.cwd();
    const env = this.env ?? parseEnv(process.env.ENV) ?? 'live';

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const config: any = await readWranglerConfig(workingDir);
    const version = await getVersion({ workingDir, env });

    config.vars = {
      ENV: env,
      VERSION: version,
    };

    const configFileName = this.getWranglerFileName(env);

    await writeWranglerConfig(workingDir, configFileName, config);

    const args = ['-c', configFileName];
    if (env === 'test') {
      args.push('--env', 'test');
    }

    await execa('wrangler', ['publish', ...args], {
      cwd: workingDir,
      stdio: 'inherit',
    });
  }

  private getWranglerFileName(env: Env) {
    switch (env) {
      case 'live':
        return '.live.wrangler.toml';
      case 'test':
        return '.test.wrangler.toml';
    }
  }
}
