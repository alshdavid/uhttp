import { h, render, Fragment } from 'preact'

export const App = (props: { title: string }) => <Fragment>
  <div>{window.location.href}</div>
  <h1>{props.title}</h1>
</Fragment>

render(<App title={"Hello World"} />, document.getElementById('root')!);
