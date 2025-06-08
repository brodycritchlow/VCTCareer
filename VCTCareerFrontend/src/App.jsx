import './App.css';
import {
  Route,
  BrowserRouter as Router,
  Routes,
  useLocation,
} from 'react-router-dom';
import Navbar from './components/common/navbar';
import Career from './pages/career/career';
import Landing from './pages/landing/landing';

const navigationItemsLanding = [{ name: 'Home', href: '/' }];
const navigationItemsCareer = [
  { name: 'Dashboard', href: '/career' },
  { name: 'Ranked', href: '/career/ranked' },
  { name: 'Leaderboard', href: '/career/leaderboard' },
  { name: 'Schedule', href: '/career/schedule' },
];
const sidebarNavigationItems = [
  { name: 'Offers', href: '/career/offers' },
  { name: 'Tryouts', href: '/career/tryouts' },
  { name: 'Sponsors', href: '/career/sponsors' },
];

const NavigationWrapper = ({ children }) => {
  const location = useLocation();
  // Show different navs based on route
  const isCareer = location.pathname.startsWith('/career');
  const navigation = (
    isCareer ? navigationItemsCareer : navigationItemsLanding
  ).map((item) => ({
    ...item,
    current: location.pathname === item.href,
  }));
  return (
    <>
      <Navbar
        navigation={navigation}
        sidebarNavigation={sidebarNavigationItems}
        hideUserProfile={true}
        logoSrc={require('./assets/logo/navlogo.png')}
        showHamburger={true}
      />
      {children}
    </>
  );
};

const App = () => {
  return (
    <Router>
      <NavigationWrapper>
        <Routes>
          <Route path="/" element={<Landing />} />
          <Route path="/career" element={<Career />} />
        </Routes>
      </NavigationWrapper>
    </Router>
  );
};

export default App;
