import { h, render, Fragment, type TargetedInputEvent } from "preact";
import { rx, useViewModel } from "rx";

export class AppViewModel {
  @rx accessor message = "";
}

export const App = (props: { title: string }) => {
  const vm = useViewModel(AppViewModel, []);

  return <Fragment>
    <div>{window.location.href}</div>
    <div>
      <div>{vm.message}</div>
      <input
        onInput={(e: TargetedInputEvent<HTMLInputElement>) =>
          (vm.message = e.currentTarget.value)
        }
        value={vm.message}
      />
    </div>
  </Fragment>
}

render(<App />, document.getElementById('root')!);
