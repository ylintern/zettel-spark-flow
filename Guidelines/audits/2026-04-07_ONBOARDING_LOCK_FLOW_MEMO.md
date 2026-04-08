# Onboarding Lock Flow Memo

## Date
- 2026-04-07

## Issue
- onboarding gravava `authMethod`
- lock/setup screen ignorava essa escolha
- resultado:
  - UX dizia uma coisa
  - fluxo real forçava PIN
  - falha de setup podia parecer bloqueio sem feedback

## Fix
- `LockScreen` agora lê `authMethod` do onboarding
- `passphrase` e `biometrics` usam fluxo canónico de passphrase
- erros de setup/unlock agora aparecem na UI
- estado de submissão evita clique duplo e falso “nada aconteceu”

## Why It Matters
- segurança e onboarding não podem divergir
- biometria continua a ser conveniência; passphrase continua a ser o fallback canónico
- UX de setup precisa refletir a arquitetura real

## Verified
- `bun run build`
- `bun run test`
