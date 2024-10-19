import { Navbar, Button, Alignment } from "@blueprintjs/core";
import { Link } from "react-router-dom";
import { useCallback, useState } from "react";
function AppNavbar() {
  const [isOpen, setIsOpen] = useState(false);
  const toggleOverlay = useCallback(
    () => setIsOpen((open) => !open),
    [setIsOpen],
  );
  return (
    <div>
      <Navbar>
        <Navbar.Group align={Alignment.LEFT}>
          <Navbar.Heading>Practice Book</Navbar.Heading>
          <Navbar.Divider />
          <Link to="/" className="bp5-button bp5-minimal">
            <Button className="bp5-minimal" icon="home" text="Home" />
          </Link>
          <Link to="/about" className="bp5-button bp5-minimal">
            <Button className="bp5-minimal" icon="info-sign" text="About" />
          </Link>
          <Link to="/practice-regiments" className="bp5-button bp5-minimal">
            <Button
              className="bp5-minimal"
              icon="document"
              text="Practice Regiments"
            />
          </Link>
          <Link
            to="/create-practice-regiment"
            className="bp5-button bp5-minimal"
          >
            <Button
              className="bp5-minimal"
              icon="document"
              text="Create Practice Regiment"
            />
          </Link>
        </Navbar.Group>
      </Navbar>
    </div>
  );
}

export default AppNavbar;
