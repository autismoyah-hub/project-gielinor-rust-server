import json, os, pathlib, time

class Gateway:

    def __init__(self, directory, log):
        self.directory = pathlib.Path(directory)
        self.log = pathlib.Path(log)
        self.sequence = 0
        self.log.parent.mkdir(parents=True, exist_ok=True)

    def read(self):
        for _ in range(20):
            try:
                return json.loads((self.directory / 'state.json').read_text(encoding='utf-8-sig'))
            except (OSError, ValueError):
                time.sleep(0.025)
        raise RuntimeError('No valid client observation')

    def act(self, action, timeout=10):
        self.sequence = max(self.sequence, self.read().get('sequence', 0)) + 1
        command = self.directory / 'gateway.tmp'
        command.write_text(f'{self.sequence} {action}\n', encoding='utf-8')
        os.replace(command, self.directory / 'command.txt')
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            s = self.read()
            if s.get('sequence') == self.sequence:
                with self.log.open('a', encoding='utf-8') as f:
                    f.write(json.dumps({'time': time.time(), 'action': action, 'state': s}, ensure_ascii=False) + '\n')
                if s.get('status') not in ('accepted', 'camera updated') and (not (action.startswith('interface ') and s.get('status') == 'interface operation unavailable')):
                    raise RuntimeError(f"Action rejected: {action}: {s.get('status')}")
                return s
            time.sleep(0.04)
        raise TimeoutError(f'Client did not acknowledge {action}')

    def observe(self):
        return self.act('observe')

    def wait(self, predicate, timeout=50):
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            s = self.observe()
            if predicate(s):
                return s
            time.sleep(0.2)
        raise TimeoutError('Authoritative observation did not reach expected state')

    def eat(self, s):
        food = next((i for i in s['items'] if i['id'] == 315 and i['count'] > 0), None)
        if food is None:
            raise RuntimeError('No cooked shrimp remain')
        op = next((n + 1 for n, label in enumerate(food['operations']) if label.lower() == 'eat'), None)
        if op is None:
            raise RuntimeError('No visible Eat operation on shrimp')
        try:
            return self.act(f"item {food['slot']} {op}")
        except RuntimeError as error:
            if 'inventory operation has no displayed action' not in str(error):
                raise
            self.interface(35913795)
            fresh=self.wait(lambda state:state.get('ready') and any(i['id']==315 and i.get('widget') is not None and 'Eat' in i.get('operations',[]) for i in state['items']),15)
            food=next(i for i in fresh['items'] if i['id']==315 and i.get('widget') is not None and 'Eat' in i.get('operations',[]))
            return self.act(f"item {food['slot']} 2")

    def interface(self, widget, operation=1, timeout=55):
        return self.act(f'interface {widget} {operation}',timeout=timeout)
