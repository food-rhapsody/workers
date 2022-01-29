import { Cli } from 'clipanion';
import { DeployCommand } from './DeployCommand';

const cli = new Cli({
  binaryName: 'fr-deploy-cli',
  binaryLabel: '푸드랩소디 배포 커맨드라인',
});

cli.register(DeployCommand);
cli.runExit(process.argv.slice(2));
