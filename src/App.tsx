import Settings from "./Settings";
import Overlay from "./Overlay";

function App() {
  const hostUrl = window.location.pathname;
  if (hostUrl === "/settings") {
    return <Settings />;
  }

  if (hostUrl === "/overlay") {
    return <Overlay />;
  }

  return (
    <div>
      <h1>Something is broken...</h1>
    </div>
  );
}

export default App;
