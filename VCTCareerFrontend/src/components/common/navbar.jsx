import {
  Disclosure,
  DisclosureButton,
  DisclosurePanel,
  Menu,
  MenuButton,
  MenuItem,
  MenuItems,
} from '@headlessui/react';
import { Menu as Bars3, Bell, X } from 'lucide-react';
import React, { useState } from 'react';

function classNames(...classes) {
  return classes.filter(Boolean).join(' ');
}

const Navbar = ({
  navigation,
  profileNavigation,
  logoSrc,
  profileIconSrc,
  hideUserProfile,
  enableHamburger = true,
  sidebarNavigation,
}) => {
  const currentNavigation = navigation;
  const currentProfileNavigation = profileNavigation;

  const finalLogoSrc =
    logoSrc || 'https://placehold.co/32x32/indigo/white?text=LOGO';
  const finalProfileIconSrc =
    profileIconSrc || 'https://placehold.co/32x32/gray/white?text=USER';
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const sidebarNav = sidebarNavigation || currentNavigation;

  return (
    <>
      <Disclosure as="nav" className="bg-gray-800 font-sans sticky top-0 z-50">
        <div className="relative flex h-16 items-center justify-between">
          {/* Hamburger (very left) */}
          {enableHamburger && (
            <div className="flex items-center pl-2 sm:pl-4 z-20">
              <button
                type="button"
                className="inline-flex items-center justify-center rounded-md p-2 text-gray-400 hover:bg-gray-700 hover:text-white focus:outline-none focus:ring-2 focus:ring-white focus:ring-inset"
                onClick={() => setSidebarOpen(true)}
                aria-label="Open sidebar menu"
              >
                <Bars3 aria-hidden="true" className="size-6" />
              </button>
            </div>
          )}
          {/* Logo (next to hamburger) */}
          <div className="flex shrink-0 items-center ml-2">
            <img
              alt="Your Company"
              src={finalLogoSrc}
              className="h-16 w-auto rounded-md"
              style={{ filter: 'invert(1)' }}
            />
          </div>
          {/* Center section: Absolutely centered navigation */}
          <div className="absolute left-0 right-0 flex justify-center pointer-events-none">
            <div className="flex space-x-4 pointer-events-auto">
              {currentNavigation?.map((item) => (
                <a
                  key={item.name}
                  href={item.href}
                  aria-current={item.current ? 'page' : undefined}
                  className={classNames(
                    item.current
                      ? 'bg-gray-900 text-white'
                      : 'text-gray-300 hover:bg-gray-700 hover:text-white',
                    'rounded-md px-3 py-2 text-sm font-medium',
                  )}
                >
                  {item.name}
                </a>
              ))}
            </div>
          </div>
          {/* Right Group */}
          {!hideUserProfile && (
            <div className="flex items-center pr-2 sm:pr-6 lg:pr-8 z-10">
              <button
                type="button"
                className="relative rounded-full bg-gray-800 p-1 text-gray-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800"
              >
                <span className="absolute -inset-1.5" />
                <span className="sr-only">View notifications</span>
                <Bell aria-hidden="true" className="size-6" />
              </button>

              <Menu as="div" className="relative ml-3">
                <div>
                  <MenuButton className="relative flex rounded-full bg-gray-800 text-sm focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800">
                    <span className="absolute -inset-1.5" />
                    <span className="sr-only">Open user menu</span>
                    <img
                      alt="User Profile"
                      src={finalProfileIconSrc}
                      className="size-8 rounded-full"
                    />
                  </MenuButton>
                </div>
                <MenuItems
                  transition
                  className="absolute right-0 z-10 mt-2 w-48 origin-top-right rounded-md bg-white py-1 shadow-lg ring-1 ring-black ring-opacity-5 transition focus:outline-none data-[closed]:scale-95 data-[closed]:transform data-[closed]:opacity-0 data-[enter]:duration-100 data-[enter]:ease-out data-[leave]:duration-75 data-[leave]:ease-in"
                >
                  {currentProfileNavigation?.map((item) => (
                    <MenuItem key={item.name}>
                      {({ focus }) => (
                        <a
                          href={item.href}
                          className={classNames(
                            focus ? 'bg-gray-100' : '',
                            'block px-4 py-2 text-sm text-gray-700',
                          )}
                        >
                          {item.name}
                        </a>
                      )}
                    </MenuItem>
                  ))}
                </MenuItems>
              </Menu>
            </div>
          )}
        </div>
        {/* Mobile Disclosure Panel (disabled for sidebar) */}
        {/* Sidebar overlay and panel */}
        {enableHamburger && sidebarOpen && (
          <div className="fixed inset-0 z-40 flex">
            {/* Overlay - just transparent, 0.9 transparency */}
            <div
              className="fixed inset-0 bg-black bg-opacity-25 transition-opacity"
              style={{ backgroundColor: 'rgba(0,0,0,0.1)' }}
              onClick={() => setSidebarOpen(false)}
              onKeyDown={(e) => {
                if (e.key === 'Escape') {
                  setSidebarOpen(false);
                }
              }}
              role="button"
              tabIndex={0}
              aria-label="Close sidebar"
            />
            {/* Sidebar */}
            <div className="relative w-64 bg-gray-900 h-full shadow-xl flex flex-col p-6 animate-slide-in-left">
              <button
                type="button"
                className="absolute top-4 right-4 text-gray-400 hover:text-white"
                onClick={() => setSidebarOpen(false)}
                aria-label="Close sidebar"
              >
                <X className="size-6" />
              </button>
              <div className="flex flex-col mt-8 space-y-2">
                {sidebarNav?.map((item) => (
                  <a
                    key={item.name}
                    href={item.href}
                    className={classNames(
                      item.current
                        ? 'bg-gray-800 text-white'
                        : 'text-gray-300 hover:bg-gray-700 hover:text-white',
                      'rounded-md px-4 py-2 text-base font-medium',
                    )}
                    onClick={() => setSidebarOpen(false)}
                  >
                    {item.name}
                  </a>
                ))}
              </div>
            </div>
            <style>{`
                        @keyframes slide-in-left {
                            from { transform: translateX(-100%); }
                            to { transform: translateX(0); }
                        }
                        .animate-slide-in-left {
                            animation: slide-in-left 0.25s cubic-bezier(0.4,0,0.2,1);
                        }
                    `}</style>
          </div>
        )}
      </Disclosure>
    </>
  );
};

export default Navbar;
