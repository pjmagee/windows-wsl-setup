export const EXE =
  'https://github.com/pjmagee/windows-wsl-manager/releases/latest/download/wwm.exe';
export const RELEASES = 'https://github.com/pjmagee/windows-wsl-manager/releases';
export const REPO = 'https://github.com/pjmagee/windows-wsl-manager';

export const INSTALL_PS = [
  'New-Item $HOME\\.wwm -ItemType Directory -Force | Out-Null',
  `Invoke-WebRequest -UseBasicParsing -Uri ${EXE} -OutFile $HOME\\.wwm\\wwm.exe`,
  '$env:Path = "$HOME\\.wwm;$env:Path"',
  'wwm',
].join('\n');
