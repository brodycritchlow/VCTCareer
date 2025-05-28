import './App.css';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Navbar from './components/common/navbar';
import Landing from './pages/landing/landing';

const mainNavigation = [
  { name: 'Home', href: '/', current: true },
];

const App = () => {
  return (
    <Router>
      <Navbar
        navigation={mainNavigation}
        hideUserProfile={true}
        logoSrc={require('./assets/logo/navlogo.png')}
      />
      <Routes>
        <Route path="/" element={<Landing />} />
      </Routes>
    </Router>
  );
};

export default App;
